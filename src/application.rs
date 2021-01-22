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
use std::path::PathBuf;
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
pub struct Application {
	current_page: Rc<Mutex<usize>>,
	pages: Rc<Mutex<Vec<Page>>>,
	application_layout: ApplicationLayout,
	area: DrawingArea,
}

impl Application {
	pub fn new() -> Self {
		if gtk::init().is_err() {
			println!("Failed to initialize GTK.");
		}
		let current_page = Rc::new(Mutex::new(0));
		let application_layout = ApplicationLayout::new();
		let area = DrawingArea::new();
		area.add_events(EventMask::ALL_EVENTS_MASK);
		Self {
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
		}
	}

	pub fn build_ui(&self, application: &gtk::Application) {
		let window = gtk::ApplicationWindow::new(application);

		window.set_title("Handwritten notetaking editor");
		window.set_border_width(10);
		window.set_position(WindowPosition::Center);
		window.set_default_size(800, 600);

		self.application_layout(&window);

		window.show_all();
	}

	fn application_layout(&self, window: &ApplicationWindow) {
		let menu_bar = self.menu_bar(&window);

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

		self.drawing_mechanics(
			&self.area,
			&self.application_layout.tool_pack,
			&self.application_layout.page_pack,
		);

		window.add(&self.application_layout.vertical_pack_0);
	}

	fn menu_bar(&self, window: &ApplicationWindow) -> MenuBar {
		let menu = Menu::new();
		let menu_bar = MenuBar::new();
		let file = MenuItem::with_label("File");
		let open_file = MenuItem::new();
		open_file.add(&Label::new(Some("Open File")));

		let pages = Rc::clone(&self.pages);
		open_file.connect_activate(clone!(@strong window => move |_| {
			let file_chooser = gtk::FileChooserDialogBuilder::new()
				.title("Choose file...")
				.action(gtk::FileChooserAction::Open)
				.local_only(false)
				.transient_for(&window)
				.modal(true)
				.build();
			let pages = Rc::clone(&pages);
			file_chooser.connect_response(move |file_chooser, response| {
				if response == gtk::ResponseType::Ok {
					let filename = file_chooser.get_filename().expect("Couldn't get filename");
					Self::load_file(&filename, &pages);
				}
				file_chooser.close();
			});

		}));

		let save_file = MenuItem::new();
		save_file.add(&Label::new(Some("Save File")));

		let pages = Rc::clone(&self.pages);
		save_file.connect_activate(clone!(@strong window => move |_| {
			let file_chooser = gtk::FileChooserDialogBuilder::new()
				.title("Choose file...")
				.action(gtk::FileChooserAction::Save)
				.local_only(false)
				.transient_for(&window)
				.modal(true)
				.build();
			let pages = Rc::clone(&pages);
			file_chooser.connect_response(move |file_chooser, response| {
				if response == gtk::ResponseType::Ok {
					let filename = file_chooser.get_filename().expect("Couldn't get filename");
					Self::save_file(&filename, &pages);
				}
				file_chooser.close();
			});

		}));

		menu.append(&open_file);
		menu.append(&save_file);
		file.set_submenu(Some(&menu));
		menu_bar.append(&file);

		menu_bar
	}

	fn drawing_mechanics(&self, area: &DrawingArea, pack: &Box, page_pack: &Box) {
		let button_press = Rc::new(Mutex::new(false));

		self.add_pages(page_pack, area, &self.pages, &self.current_page);

		self.save_file(pack, &self.pages);

		self.undo_redo(pack, area, &self.pages, &self.current_page);

		let size_tool = Scale::with_range(Orientation::Horizontal, 0.5, 50.0, 0.5);
		pack.pack_start(&size_tool, true, true, 0);

		self.manage_drawing_modes(
			pack,
			area,
			&self.pages,
			&self.current_page,
			&button_press,
			&size_tool,
		);

		{
			let pages = Rc::clone(&self.pages);
			let current_page = Rc::clone(&self.current_page);
			let button_press = Rc::clone(&button_press);
			area.connect_button_press_event(move |_, _| {
				let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
				*button_press.lock().unwrap() = true;
				lines.push(Vec::new());
				Inhibit(false)
			});
		}
		{
			let button_press = Rc::clone(&button_press);
			area.connect_button_release_event(move |_, _| {
				*button_press.lock().unwrap() = false;
				Inhibit(false)
			});
		}
		{
			let pages = Rc::clone(&self.pages);
			let current_page = Rc::clone(&self.current_page);
			area.connect_draw(move |_, cr| {
				let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
				cr.set_source_rgb(1.0, 1.0, 1.0);
				cr.paint();
				for stroke in lines.iter() {
					for drawpoint in stroke {
						cr.set_source_rgba(
							drawpoint.rgba.0,
							drawpoint.rgba.1,
							drawpoint.rgba.2,
							drawpoint.rgba.3,
						);
						cr.set_line_width(drawpoint.line_width);
						cr.line_to(drawpoint.position.0, drawpoint.position.1);
					}
					cr.stroke();
				}
				Inhibit(false)
			});
		}

		self.position_pointer(area, &size_tool);
	}

	fn save_file(pack: &Box, pages: &Rc<Mutex<Vec<Page>>>) {
		let pages = pages.lock().unwrap();
		let serialized = serde_json::to_string(&pages.clone()).unwrap();
		let mut file = File::create("test.hnote").unwrap();
		file.write_all(serialized.as_bytes());
	}

	fn load_file(path_puf: &PathBuf, pages: &Rc<Mutex<Vec<Page>>>) {
		let mut file = File::open(path_puf).expect("Could not open file.");
		let mut serialized = std::string::String::new();
		file.read_to_string(&mut serialized)
			.expect("Could not read to string.");
		*pages.lock().unwrap() = serde_json::from_str(&serialized).expect("Invalid format.");
	}

	fn undo_redo(
		&self,
		pack: &Box,
		area: &DrawingArea,
		pages: &Rc<Mutex<Vec<Page>>>,
		current_page: &Rc<Mutex<usize>>,
	) {
		let removed_lines = Rc::new(Mutex::new(Vec::<Vec<Drawpoint>>::new()));
		let undo = Button::with_label("Undo");
		{
			let pages = Rc::clone(&pages);
			let current_page = Rc::clone(&current_page);
			let removed_lines = Rc::clone(&removed_lines);
			undo.connect_clicked(clone!(@strong area => move |_| {
				let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
				let removed_lines = &mut removed_lines.lock().unwrap();
				if !lines.is_empty() {
					removed_lines.push(lines.pop().unwrap());
					area.queue_draw();
				}
			}));
		}
		pack.pack_start(&undo, false, false, 0);

		let redo = Button::with_label("Redo");
		{
			let pages = Rc::clone(&pages);
			let current_page = Rc::clone(&current_page);
			let removed_lines = Rc::clone(&removed_lines);
			redo.connect_clicked(clone!(@strong area => move |_| {
				let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
				let removed_lines = &mut removed_lines.lock().unwrap();
				if !removed_lines.is_empty() {
					lines.push(removed_lines.pop().unwrap());
					area.queue_draw();
				}
			}));
		}
		pack.pack_start(&redo, false, false, 0);
	}

	fn add_pages(
		&self,
		page_pack: &Box,
		area: &DrawingArea,
		pages: &Rc<Mutex<Vec<Page>>>,
		current_page: &Rc<Mutex<usize>>,
	) {
		let add_page = Button::with_label("+");
		{
			let pages = Rc::clone(&pages);
			let current_page = Rc::clone(&current_page);
			add_page.connect_clicked(clone!(@strong area, @strong page_pack => move |_| {
				let page = Page::new(
					Rc::clone(&current_page),
					area.clone(),
					&page_pack,
					pages.lock().unwrap().len(),
				);
				let pages = &mut pages.lock().unwrap();
				println!("{:?}", page_pack.get_children());
				pages.push(page);
				page_pack.queue_draw();
			}));
		}
		page_pack.pack_start(&add_page, false, false, 0);
	}

	fn manage_drawing_modes(
		&self,
		pack: &Box,
		area: &DrawingArea,
		pages: &Rc<Mutex<Vec<Page>>>,
		current_page: &Rc<Mutex<usize>>,
		button_press: &Rc<Mutex<bool>>,
		size_tool: &Scale,
	) {
		let drawing_alpha = Scale::with_range(Orientation::Horizontal, 0.01, 1.0, 0.01);
		pack.pack_start(&drawing_alpha, true, true, 0);

		let current_draw_tool = Rc::new(Mutex::new(CurrentDrawTool::Pencil));
		let pencil = Rc::new(Mutex::new(Heap::new(Pencil::new(
			Rc::clone(&current_draw_tool),
			pack,
		))));
		let eraser = Rc::new(Mutex::new(Heap::new(Eraser::new(
			Rc::clone(&current_draw_tool),
			pack,
		))));
		let line_eraser = Rc::new(Mutex::new(Heap::new(LineEraser::new(
			Rc::clone(&current_draw_tool),
			pack,
		))));
		let line = Rc::new(Mutex::new(Heap::new(LineTool::new(
			Rc::clone(&current_draw_tool),
			pack,
		))));
		let selection = Rc::new(Mutex::new(Heap::new(Selection::new(
			Rc::clone(&current_draw_tool),
			pack,
		))));

		let pages = Rc::clone(&pages);
		let current_page = Rc::clone(&current_page);
		let button_press = Rc::clone(&button_press);
		area.connect_motion_notify_event(clone!(@strong area, @strong size_tool => move |_, e| {
		if *button_press.lock().unwrap() {
			let active_draw_tool: Heap<dyn DrawTool> = match *current_draw_tool.lock().unwrap() {
				CurrentDrawTool::Pencil => Heap::new(*Rc::clone(&pencil).lock().unwrap().clone()),
				CurrentDrawTool::Eraser => Heap::new(*Rc::clone(&eraser).lock().unwrap().clone()),
				CurrentDrawTool::LineEraser => Heap::new(*Rc::clone(&line_eraser).lock().unwrap().clone()),
				CurrentDrawTool::LineTool => Heap::new(*Rc::clone(&line).lock().unwrap().clone()),
				CurrentDrawTool::Selection => Heap::new(*Rc::clone(&selection).lock().unwrap().clone()),
			};
			active_draw_tool.manipulate(Rc::clone(&pages), Rc::clone(&current_page), e.get_position(), size_tool.get_value(), drawing_alpha.get_value());
		}
		area.queue_draw();
		Inhibit(false)
	}));
	}

	fn position_pointer(&self, area: &DrawingArea, size_tool: &Scale) {
		let cursor_position = Rc::new(Mutex::new(Some((0.0, 0.0))));

		{
			let cursor_position = Rc::clone(&cursor_position);
			area.connect_motion_notify_event(clone!(@strong area => move |_, e| {
				*cursor_position.lock().unwrap() = Some(e.get_position());
				Inhibit(false)
			}));
		}
		{
			let cursor_position = Rc::clone(&cursor_position);
			area.connect_leave_notify_event(move |_, _| {
				*cursor_position.lock().unwrap() = None;
				Inhibit(false)
			});
		}
		{
			let cursor_position = Rc::clone(&cursor_position);
			area.connect_draw(clone!(@strong size_tool => move |_, cr| {
			let cursor_position = *cursor_position.lock().unwrap();
			if cursor_position.is_some() {
				cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
				cr.set_line_width(5.0);
				cr.arc(cursor_position.unwrap().0, cursor_position.unwrap().1, size_tool.get_value() / 2.0, 0.0, PI * 2.0);
				cr.stroke();
			}
			Inhibit(false)
		}));
		}
	}
}
