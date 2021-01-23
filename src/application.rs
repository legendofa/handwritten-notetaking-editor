use crate::datatypes::*;
use gdk::*;
use glib::*;
use gtk::prelude::*;
use gtk::*;
use serde_json::*;
use std::boxed::Box as Heap;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Mutex;

#[derive(Clone, Debug)]
struct ApplicationLayout {
	pub vertical_pack_0: Box,
	pub page_pack: Box,
	pub tool_pack: Box,
	pub horizontal_pack_1: Box,
}

impl ApplicationLayout {
	pub fn new() -> Self {
		Self {
			vertical_pack_0: Box::new(Orientation::Vertical, 0),
			page_pack: Box::new(Orientation::Vertical, 0),
			tool_pack: Box::new(Orientation::Horizontal, 0),
			horizontal_pack_1: Box::new(Orientation::Horizontal, 0),
		}
	}
}

#[derive(Clone, Debug)]
struct DrawingInformation {
	rgba: Rc<Mutex<[f64; 4]>>,
	pen_is_active: Rc<Mutex<bool>>,
	pen_size: Rc<Mutex<f64>>,
	cursor_position: Rc<Mutex<Option<(f64, f64)>>>,
}

impl DrawingInformation {
	pub fn new() -> Self {
		Self {
			rgba: Rc::new(Mutex::new([0.0, 0.0, 0.0, 1.0])),
			pen_is_active: Rc::new(Mutex::new(false)),
			pen_size: Rc::new(Mutex::new(25.0)),
			cursor_position: Rc::new(Mutex::new(Some((0.0, 0.0)))),
		}
	}
}

#[derive(Clone, Debug)]
pub struct Application {
	current_page: Rc<Mutex<usize>>,
	pages: Rc<Mutex<Vec<Page>>>,
	application_layout: ApplicationLayout,
	area: DrawingArea,
	drawing_information: DrawingInformation,
	window: ApplicationWindow,
}

impl Application {
	pub fn new(gtk_application: &gtk::Application) -> Self {
		let window = gtk::ApplicationWindow::new(gtk_application);
		let current_page = Rc::new(Mutex::new(0));
		let application_layout = ApplicationLayout::new();
		let area = DrawingArea::new();
		area.add_events(EventMask::ALL_EVENTS_MASK);
		let drawing_information = DrawingInformation::new();
		let application = Self {
			current_page: Rc::clone(&current_page),
			pages: Rc::new(Mutex::new(vec![
				Page::new(
					Rc::clone(&current_page),
					area.clone(),
					&application_layout.page_pack,
					0,
				),
				Page::new(current_page, area.clone(), &application_layout.page_pack, 1),
			])),
			application_layout,
			area,
			drawing_information,
			window: window.clone(),
		};
		application.build_ui();
		application
	}

	pub fn build_ui(&self) {
		self.window.set_title("Handwritten notetaking editor");
		self.window.set_border_width(10);
		self.window.set_position(WindowPosition::Center);
		self.window.set_default_size(800, 600);

		self.application_layout(&self.window);

		self.window.show_all();
	}

	fn application_layout(&self, window: &ApplicationWindow) {
		let menu_bar = self.menu_bar();

		self.application_layout
			.vertical_pack_0
			.pack_start(&menu_bar, false, false, 0);
		self.application_layout.vertical_pack_0.pack_start(
			&self.application_layout.tool_pack,
			false,
			false,
			0,
		);
		self.application_layout.vertical_pack_0.pack_start(
			&self.application_layout.horizontal_pack_1,
			true,
			true,
			0,
		);

		self.application_layout.horizontal_pack_1.pack_start(
			&self.application_layout.page_pack,
			false,
			false,
			0,
		);
		self.application_layout
			.horizontal_pack_1
			.pack_start(&self.area, true, true, 0);

		self.drawing_mechanics();

		window.add(&self.application_layout.vertical_pack_0);
	}

	fn menu_bar(&self) -> MenuBar {
		let menu = Menu::new();
		let menu_bar = MenuBar::new();
		let file = MenuItem::with_label("File");
		let open_file = MenuItem::new();
		open_file.add(&Label::new(Some("Open File")));

		open_file.connect_activate(clone!(@strong self as this => move |_| {
			let file_chooser = gtk::FileChooserDialogBuilder::new()
				.title("Choose file...")
				.action(gtk::FileChooserAction::Open)
				.local_only(false)
				.transient_for(&this.window)
				.modal(true)
				.build();
			file_chooser.connect_response(clone!(@strong this => move |file_chooser, response| {
				if response == gtk::ResponseType::Ok {
					let filename = file_chooser.get_filename().expect("Couldn't get filename");
					this.load_file(&filename);
				}
				file_chooser.close();
			}));
		}));

		let save_file = MenuItem::new();
		save_file.add(&Label::new(Some("Save File")));
		save_file.connect_activate(clone!(@strong self as this => move |_| {
			let file_chooser = gtk::FileChooserDialogBuilder::new()
				.title("Save file...")
				.action(gtk::FileChooserAction::Save)
				.local_only(false)
				.transient_for(&this.window)
				.modal(true)
				.build();
			file_chooser.connect_response(clone!(@strong this => move |file_chooser, response| {
				if response == gtk::ResponseType::Ok {
					let filename = file_chooser.get_filename().expect("Couldn't get filename");
					this.save_file(&filename);
				}
				file_chooser.close();
			}));
		}));

		menu.append(&open_file);
		menu.append(&save_file);
		file.set_submenu(Some(&menu));
		menu_bar.append(&file);

		menu_bar
	}

	fn drawing_mechanics(&self) {
		self.add_pages();

		self.undo_redo();

		let save = Button::with_label("Save");
		{
			save.connect_clicked(clone!(@strong self as this => move |_| {
				this.save_file(&Path::new("test.hnote").to_path_buf());
			}));
		}
		self.application_layout
			.tool_pack
			.pack_start(&save, false, false, 0);

		let load = Button::with_label("Load");
		{
			load.connect_clicked(clone!(@strong self as this => move |_| {
				this.load_file(&Path::new("test.hnote").to_path_buf());
			}));
		}
		self.application_layout
			.tool_pack
			.pack_start(&load, false, false, 0);

		let pen_size = Scale::with_range(Orientation::Horizontal, 0.5, 50.0, 0.5);
		pen_size.set_value(*self.drawing_information.pen_size.lock().unwrap());
		pen_size.connect_change_value(
			clone!(@strong self.drawing_information.pen_size as pen_size => move |_, _, v| {
				*pen_size.lock().unwrap() = v;
				Inhibit(false)
			}),
		);

		self.application_layout
			.tool_pack
			.pack_start(&pen_size, true, true, 0);

		self.manage_drawing_modes();

		{
			self.area
				.connect_button_press_event(clone!(@strong self as this => move |_, _| {
					let lines = &mut this.pages.lock().unwrap()[*this.current_page.lock().unwrap()].lines;
					*this.drawing_information.pen_is_active.lock().unwrap() = true;
					lines.push(Vec::new());
					Inhibit(false)
				}));
		}
		{
			self.area.connect_button_release_event(
				clone!(@strong self.drawing_information.pen_is_active as pen_is_active => move |_, _| {
					*pen_is_active.lock().unwrap() = false;
					Inhibit(false)
				}),
			);
		}
		{
			self.area
				.connect_draw(clone!(@strong self as this => move |_, cr| {
					let lines = &mut this.pages.lock().unwrap()[*this.current_page.lock().unwrap()].lines;
					cr.set_source_rgb(1.0, 1.0, 1.0);
					cr.paint();
					for stroke in lines.iter() {
						for drawpoint in stroke {
							cr.set_source_rgba(
								drawpoint.rgba[0],
								drawpoint.rgba[1],
								drawpoint.rgba[2],
								drawpoint.rgba[3],
							);
							cr.set_line_width(drawpoint.line_width);
							cr.line_to(drawpoint.position.0, drawpoint.position.1);
						}
						cr.stroke();
					}
					Inhibit(false)
				}));
		}

		self.position_pointer();
	}

	fn save_file(&self, path_puf: &PathBuf) {
		let pages = self.pages.lock().unwrap();
		let serialized = serde_json::to_string(&pages.clone()).expect("Could not serialize pages.");
		let mut file = File::create(path_puf).expect("Could not create file.");
		file.write_all(serialized.as_bytes());
	}

	fn load_file(&self, path_puf: &PathBuf) {
		let mut file = File::open(path_puf).expect("Could not open file.");
		let mut serialized = std::string::String::new();
		file.read_to_string(&mut serialized)
			.expect("Could not read to string.");
		*self.pages.lock().unwrap() = serde_json::from_str(&serialized).expect("Invalid format.");
		for button in self.application_layout.page_pack.get_children() {
			self.application_layout.page_pack.remove(&button);
		}
		let pages = self.pages.lock().unwrap();
		for (i, _) in pages.iter().enumerate() {
			Page::connect_pack(
				Rc::clone(&self.current_page),
				self.area.clone(),
				&self.application_layout.page_pack,
				i,
			);
		}
		self.add_pages();
		self.application_layout.page_pack.show_all();
	}

	fn undo_redo(&self) {
		let removed_lines = Rc::new(Mutex::new(Vec::<Vec<Drawpoint>>::new()));
		let undo = Button::with_label("Undo");
		{
			undo.connect_clicked(
				clone!(@strong self as this, @strong removed_lines => move |_| {
					let lines = &mut this.pages.lock().unwrap()[*this.current_page.lock().unwrap()].lines;
					let removed_lines = &mut removed_lines.lock().unwrap();
					if !lines.is_empty() {
						removed_lines.push(lines.pop().unwrap());
						this.area.queue_draw();
					}
				}),
			);
		}
		self.application_layout
			.tool_pack
			.pack_start(&undo, false, false, 0);

		let redo = Button::with_label("Redo");
		{
			redo.connect_clicked(
				clone!(@strong self as this, @strong removed_lines => move |_| {
					let lines = &mut this.pages.lock().unwrap()[*this.current_page.lock().unwrap()].lines;
					let removed_lines = &mut removed_lines.lock().unwrap();
					if !removed_lines.is_empty() {
						lines.push(removed_lines.pop().unwrap());
						this.area.queue_draw();
					}
				}),
			);
		}
		self.application_layout
			.tool_pack
			.pack_start(&redo, false, false, 0);
	}

	fn add_pages(&self) {
		let add_page = Button::with_label("+");
		{
			add_page.connect_clicked(clone!(@strong self as this => move |_| {
				let page = Page::new(
					Rc::clone(&this.current_page),
					this.area.clone(),
					&this.application_layout.page_pack,
					this.pages.lock().unwrap().len(),
				);
				let pages = &mut this.pages.lock().unwrap();
				pages.push(page);
				this.application_layout.page_pack.show_all();
			}));
		}
		self.application_layout
			.page_pack
			.pack_start(&add_page, false, false, 0);
	}

	fn manage_drawing_modes(&self) {
		let color_widget = Button::with_label("Colors");
		{
			color_widget.connect_clicked(clone!(@strong self as this => move |_| {
				let rgba = &this.drawing_information.rgba;
				let dialog = gtk::Dialog::with_buttons(
					Some("Colors"),
					Some(&this.window.clone()),
					gtk::DialogFlags::DESTROY_WITH_PARENT,
					&[("Close", ResponseType::Close)],
				);
				dialog.set_default_response(ResponseType::Close);
				dialog.connect_response(|dialog, _| dialog.close());

				let scales = [
					Scale::with_range(Orientation::Horizontal, 0.01, 1.0, 0.01),
					Scale::with_range(Orientation::Horizontal, 0.01, 1.0, 0.01),
					Scale::with_range(Orientation::Horizontal, 0.01, 1.0, 0.01),
					Scale::with_range(Orientation::Horizontal, 0.01, 1.0, 0.01),
				];
				let content_area = dialog.get_content_area();
				let color_preview = Label::new(None);
				content_area.add(&color_preview);
				for (i, scale) in scales.iter().enumerate() {
					scale.set_value(1.0);
					content_area.add(scale);
					{
						scale.connect_change_value(clone!(@strong color_preview, @strong rgba => move |_, _, v| {
							let rgba = &mut rgba.lock().unwrap();
							rgba[i] = v;
							let rgba = Some(RGBA {red: rgba[0], green: rgba[1], blue: rgba[2], alpha: rgba[3]});
							color_preview.override_background_color(StateFlags::NORMAL, rgba.as_ref());
							Inhibit(false)
						}));
					}
				}

				dialog.show_all();
			}));
		}
		self.application_layout
			.tool_pack
			.pack_start(&color_widget, false, false, 0);

		let current_draw_tool = Rc::new(Mutex::new(CurrentDrawTool::Pencil));
		let pencil = Rc::new(Mutex::new(Heap::new(Pencil::new(
			Rc::clone(&current_draw_tool),
			&self.application_layout.tool_pack,
		))));
		let eraser = Rc::new(Mutex::new(Heap::new(Eraser::new(
			Rc::clone(&current_draw_tool),
			&self.application_layout.tool_pack,
		))));
		let line_eraser = Rc::new(Mutex::new(Heap::new(LineEraser::new(
			Rc::clone(&current_draw_tool),
			&self.application_layout.tool_pack,
		))));
		let line = Rc::new(Mutex::new(Heap::new(LineTool::new(
			Rc::clone(&current_draw_tool),
			&self.application_layout.tool_pack,
		))));
		let selection = Rc::new(Mutex::new(Heap::new(Selection::new(
			Rc::clone(&current_draw_tool),
			&self.application_layout.tool_pack,
		))));

		{
			self.area.connect_motion_notify_event(clone!(@strong self as this => move |_, e| {
				if *this.drawing_information.pen_is_active.lock().unwrap() {
					let active_draw_tool: Heap<dyn DrawTool> = match *current_draw_tool.lock().unwrap() {
						CurrentDrawTool::Pencil => Heap::new(*Rc::clone(&pencil).lock().unwrap().clone()),
						CurrentDrawTool::Eraser => Heap::new(*Rc::clone(&eraser).lock().unwrap().clone()),
						CurrentDrawTool::LineEraser => Heap::new(*Rc::clone(&line_eraser).lock().unwrap().clone()),
						CurrentDrawTool::LineTool => Heap::new(*Rc::clone(&line).lock().unwrap().clone()),
						CurrentDrawTool::Selection => Heap::new(*Rc::clone(&selection).lock().unwrap().clone()),
					};
					let rgba = *this.drawing_information.rgba.lock().unwrap();
					let pen_size = *this.drawing_information.pen_size.lock().unwrap();
					active_draw_tool.manipulate(Rc::clone(&this.pages), Rc::clone(&this.current_page), e.get_position(), pen_size, rgba);
				}
				this.area.queue_draw();
				Inhibit(false)
			}));
		}
	}

	fn position_pointer(&self) {
		{
			self.area.connect_motion_notify_event(
				clone!(@strong self.drawing_information.cursor_position as cursor_position => move |_, e| {
					*cursor_position.lock().unwrap() = Some(e.get_position());
					Inhibit(false)
				}),
			);
		}
		{
			self.area.connect_leave_notify_event(
				clone!(@strong self.drawing_information.cursor_position as cursor_position => move |_, _| {
					*cursor_position.lock().unwrap() = None;
					Inhibit(false)
				}),
			);
		}
		{
			self.area
				.connect_draw(clone!(@strong self as this => move |_, cr| {
					let cursor_position = *this.drawing_information.cursor_position.lock().unwrap();
					if cursor_position.is_some() {
						let pen_size = *this.drawing_information.pen_size.lock().unwrap();
						let rgba = *this.drawing_information.rgba.lock().unwrap();
						cr.set_source_rgba(
							rgba[0],
							rgba[1],
							rgba[2],
							rgba[3],
						);
						cr.set_line_width(5.0);
						cr.arc(cursor_position.unwrap().0, cursor_position.unwrap().1, pen_size / 2.0, 0.0, PI * 2.0);
						cr.stroke();
					}
					Inhibit(false)
				}));
		}
	}
}
