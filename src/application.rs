use crate::datatypes::*;
use cairo::{Context, Format, ImageSurface};
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

/// Contains all top level groups of GTK widgets.
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

/// Contains information associated with direct drawing.
#[derive(Clone, Debug)]
struct DrawingInformation {
	rgba: Rc<Mutex<[f64; 4]>>,
	pen_is_active: Rc<Mutex<bool>>,
	pen_size: Rc<Mutex<f64>>,
	cursor_position: Rc<Mutex<Option<(f64, f64)>>>,
	current_draw_tool: Rc<Mutex<CurrentDrawTool>>,
}

impl DrawingInformation {
	pub fn new() -> Self {
		Self {
			rgba: Rc::new(Mutex::new([0.0, 0.0, 0.0, 1.0])),
			pen_is_active: Rc::new(Mutex::new(false)),
			pen_size: Rc::new(Mutex::new(25.0)),
			cursor_position: Rc::new(Mutex::new(Some((0.0, 0.0)))),
			current_draw_tool: Rc::new(Mutex::new(CurrentDrawTool::Pencil)),
		}
	}
}

/// Implements Handwritten Notetaking Editor's setup, data and logic.
#[derive(Clone, Debug)]
pub struct Application {
	current_page: Rc<Mutex<usize>>,
	pages: Rc<Mutex<Vec<Page>>>,
	pages_history: Rc<Mutex<Vec<Vec<Page>>>>,
	undone_pages_history: Rc<Mutex<Vec<Vec<Page>>>>,
	application_layout: ApplicationLayout,
	area: DrawingArea,
	drawing_information: DrawingInformation,
	window: ApplicationWindow,
	current_path: Rc<Mutex<Option<PathBuf>>>,
	image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
}

impl Application {
	/// Initializes all necessary values and instantiates the `gtk::Application`.
	pub fn new(gtk_application: &gtk::Application) -> Self {
		let window = gtk::ApplicationWindow::new(gtk_application);
		let current_page = Rc::new(Mutex::new(0));
		let application_layout = ApplicationLayout::new();
		let area = DrawingArea::new();
		area.add_events(EventMask::ALL_EVENTS_MASK);
		let drawing_information = DrawingInformation::new();
		let image_buffer = Rc::new(Mutex::new(Vec::<BufferedImage>::new()));
		let pages = Rc::new(Mutex::new(vec![Page::new(
			Rc::clone(&current_page),
			Rc::clone(&image_buffer),
			area.clone(),
			&application_layout.page_pack,
		)]));
		let pages_history = Rc::new(Mutex::new(vec![pages.lock().unwrap().clone()]));
		let undone_pages_history = Rc::new(Mutex::new(Vec::<Vec<Page>>::new()));
		let current_path = Rc::new(Mutex::new(None));
		let application = Self {
			current_page,
			pages,
			pages_history,
			undone_pages_history,
			application_layout,
			area,
			drawing_information,
			window: window.clone(),
			current_path,
			image_buffer,
		};
		application.build_ui();
		application
	}

	/// Sets basic `self.window` variables and shows the `gtk::ApplicationWindow`.
	///
	/// Invokes `self.application_layout()`.
	pub fn build_ui(&self) {
		self.window.set_title("Handwritten notetaking editor");
		self.window.set_border_width(10);
		self.window.set_position(WindowPosition::Center);
		self.window.set_default_size(800, 600);

		self.application_layout(&self.window);

		self.window.show_all();
	}

	/// Composes all top level groups to `self.window`.
	///
	/// Invokes `self.drawing_mechanics()`.
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

	/// Composes a `gtk::MenuBar` with basic export and import functionality.
	fn menu_bar(&self) -> MenuBar {
		let menu = Menu::new();
		let menu_bar = MenuBar::new();
		let file = MenuItem::with_label("File");
		let open_file = MenuItem::new();
		open_file.add(&Label::new(Some("Open...")));

		open_file.connect_activate(clone!(@strong self as this => move |_| {
			this.connect_file_dialog(FileChooserAction::Open, Heap::new(clone!(@strong this => move |current_path| {
				this.load_file(&current_path);
				*this.current_path.lock().unwrap() = Some(current_path.clone());
			})));
		}));

		let save_file = MenuItem::new();
		save_file.add(&Label::new(Some("Save...")));
		save_file.connect_activate(clone!(@strong self as this => move |_| {
			this.connect_path_or_file_dialog(FileChooserAction::Save, Heap::new(clone!(@strong this => move |current_path| {
				this.save_file(&current_path);
				*this.current_path.lock().unwrap() = Some(current_path.clone());
			})));
		}));

		let save_as_file = MenuItem::new();
		save_as_file.add(&Label::new(Some("Save as...")));
		save_as_file.connect_activate(clone!(@strong self as this => move |_| {
			this.connect_file_dialog(FileChooserAction::Save, Heap::new(clone!(@strong this => move |current_path| {
				this.save_file(&current_path);
				*this.current_path.lock().unwrap() = Some(current_path.clone());
			})));
		}));

		let import_png = MenuItem::new();
		import_png.add(&Label::new(Some("Import png...")));
		import_png.connect_activate(clone!(@strong self as this => move |_| {
			this.connect_file_dialog(FileChooserAction::Open, Heap::new(clone!(@strong this => move |current_path| {
				let mut pages = this.pages.lock().unwrap();
				let current_page = this.current_page.lock().unwrap();
				let images = &mut pages[*current_page].images;
				let mut images = images.lock().unwrap();
				let mut image_buffer = this.image_buffer.lock().unwrap();
				let initial_position = (20.0, 20.0);
				let image = Rc::new(Mutex::new(crate::datatypes::Image::new(current_path.clone(), initial_position)));
				images.push(Rc::clone(&image));
				println!("{:?}", images);
				let mut file = File::open(current_path).expect("Could not open file.");
				let image_surface =
					ImageSurface::create_from_png(&mut file).expect("Could not create ImageSurface.");
				let buffered_image = BufferedImage::new(image_surface, Rc::clone(&image));
				image_buffer.push(buffered_image);
			})));
		}));

		let export_png = MenuItem::new();
		export_png.add(&Label::new(Some("Export as png...")));
		export_png.connect_activate(clone!(@strong self as this => move |_| {
			this.export_png();
		}));

		menu.append(&open_file);
		menu.append(&save_file);
		menu.append(&save_as_file);
		menu.append(&import_png);
		menu.append(&export_png);
		file.set_submenu(Some(&menu));
		menu_bar.append(&file);

		menu_bar
	}

	/// Withholds all drawing specific methods and variables.
	///
	/// Connects basic canvas input and drawing.
	///
	/// Invokes `self.add_page()`, `self.remove_page()`, `self.undo_redo()`, `self.manage_drawing_modes()`, `self.position_pointer()`.
	fn drawing_mechanics(&self) {
		self.add_page();
		self.remove_page();
		self.move_page();

		self.undo_redo();

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

		self.area
			.connect_button_press_event(clone!(@strong self as this => move |_, _| {
				let mut pages = this.pages.lock().unwrap();
				let current_page = this.current_page.lock().unwrap();
				let lines = &mut pages[*current_page].lines;
				let mut pen_is_active = this.drawing_information.pen_is_active.lock().unwrap();
				*pen_is_active = true;
				lines.push(Vec::new());
				Inhibit(false)
			}));

		self.area.connect_button_release_event(
			clone!(@strong self.drawing_information.pen_is_active as pen_is_active => move |_, _| {
				*pen_is_active.lock().unwrap() = false;
				Inhibit(false)
			}),
		);

		self.area
			.connect_draw(clone!(@strong self as this => move |_, cr| {
				this.context_drawing_mechanics(cr);
				Inhibit(false)
			}));

		self.manage_drawing_modes();

		self.position_pointer();
	}

	/// Specifies how a context is composed out of `self`.
	fn context_drawing_mechanics(&self, cr: &Context) {
		let mut pages = self.pages.lock().unwrap();
		let current_page = self.current_page.lock().unwrap();
		let lines = &mut pages[*current_page].lines;
		let image_buffer = self.image_buffer.lock().unwrap();
		cr.set_source_rgb(1.0, 1.0, 1.0);
		cr.paint();
		for buffered_image in image_buffer.iter() {
			let image = buffered_image.image.lock().unwrap();
			cr.set_source_surface(
				&buffered_image.image_surface,
				image.position.0,
				image.position.1,
			);
			cr.paint();
		}
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
	}

	/// Saves `self` in a JSON formatted file.
	///
	/// The file path is declared by `path_puf`.
	fn save_file(&self, path_puf: &PathBuf) {
		let pages = self.pages.lock().unwrap();
		let serialized = serde_json::to_string(&pages.clone()).expect("Could not serialize pages.");
		let mut file = File::create(path_puf).expect("Could not create file.");
		file.write_all(serialized.as_bytes());
	}

	/// Loads `self` from a JSON formatted file.
	///
	/// The file path is declared by `path_puf`.
	fn load_file(&self, path_puf: &PathBuf) {
		{
			let mut pages = self.pages.lock().unwrap();
			let mut current_page = self.current_page.lock().unwrap();
			let mut image_buffer = self.image_buffer.lock().unwrap();
			let mut pages_history = self.pages_history.lock().unwrap();
			let mut undone_pages_history = self.undone_pages_history.lock().unwrap();
			let mut file = File::open(path_puf).expect("Could not open file.");
			let mut serialized = std::string::String::new();
			file.read_to_string(&mut serialized)
				.expect("Could not read to string.");
			*pages = serde_json::from_str(&serialized).expect("Invalid format.");
			*current_page = 0;
			*pages_history = vec![pages.clone()];
			undone_pages_history.clear();
			let images = &mut pages[*current_page].images;
			let images = images.lock().unwrap();
			image_buffer.clear();
			for image in images.iter() {
				let image_surface = {
					let image = image.lock().unwrap();
					let mut path = File::open(image.path.clone()).expect("Could not open file.");
					ImageSurface::create_from_png(&mut path)
						.expect("Could not create ImageSurface.")
				};
				let buffered_image = BufferedImage::new(image_surface, Rc::clone(image));
				image_buffer.push(buffered_image);
			}
		}
		self.reload_page_pack();
	}

	/// Connects a `gtk::FileChooserNative` instance with an `action`.
	fn connect_file_dialog(
		&self,
		file_chooser_action: FileChooserAction,
		action: Heap<dyn Fn(PathBuf)>,
	) {
		let file_chooser = FileChooserNativeBuilder::new()
			.title("Choose file...")
			.action(file_chooser_action)
			.local_only(false)
			.transient_for(&self.window)
			.modal(true)
			.build();
		file_chooser.connect_response(
			clone!(@strong self as this => move |file_chooser, response| {
				if response == ResponseType::Accept {
					let filename = file_chooser.get_filename().expect("Couldn't get filename.");
					action(filename);
				}
				file_chooser.destroy();
			}),
		);
		file_chooser.run();
	}

	/// Connects a `gtk::FileChooserNative` instance with an `action`.
	/// If `self.current_path` is not already specified, the `action` is executed.
	fn connect_path_or_file_dialog(
		&self,
		file_chooser_action: FileChooserAction,
		action: Heap<dyn Fn(PathBuf)>,
	) {
		let current_path = self.current_path.lock().unwrap().clone();
		match current_path {
			Some(current_path) => action(current_path.clone()),
			None => {
				self.connect_file_dialog(file_chooser_action, action);
			}
		}
	}

	/// Reloads `self.application_layout.page_pack` depending on `self`.
	fn reload_page_pack(&self) {
		for button in self.application_layout.page_pack.get_children() {
			self.application_layout.page_pack.remove(&button);
		}
		let pages = self.pages.lock().unwrap();
		for page in pages.iter() {
			page.connect_pack(
				Rc::clone(&self.current_page),
				Rc::clone(&self.image_buffer),
				self.area.clone(),
				&self.application_layout.page_pack,
			);
		}
		self.add_page();
		self.remove_page();
		self.move_page();
		self.application_layout.page_pack.show_all();
	}

	/// Implements basic version control.
	///
	/// A version is saved after each interaction with a `DrawTool`.
	fn undo_redo(&self) {
		self.area
			.connect_button_release_event(clone!(@strong self as this => move |_, _| {
				let pages = this.pages.lock().unwrap();
				let mut pages_history = this.pages_history.lock().unwrap();
				let mut undone_pages_history = this.undone_pages_history.lock().unwrap();
				pages_history.push(pages.clone());
				undone_pages_history.clear();
				Inhibit(false)
			}));

		let undo = Button::with_label("Undo");
		undo.connect_clicked(clone!(@strong self as this => move |_| {
			let mut pages_history = this.pages_history.lock().unwrap();
			if pages_history.len() > 1 {
				{
					let mut pages = this.pages.lock().unwrap();
					let mut current_page = this.current_page.lock().unwrap();
					let mut undone_pages_history = this.undone_pages_history.lock().unwrap();
					undone_pages_history.push(pages_history.pop().unwrap());
					*pages = pages_history.last().unwrap().clone();
					if *current_page > pages.len() - 1 {
						*current_page = pages.len() - 1;
					}
				}
				this.reload_page_pack();
				this.area.queue_draw();
			}
		}));
		self.application_layout
			.tool_pack
			.pack_start(&undo, false, false, 0);

		let redo = Button::with_label("Redo");
		redo.connect_clicked(clone!(@strong self as this => move |_| {
			let mut undone_pages_history = this.undone_pages_history.lock().unwrap();
			if !undone_pages_history.is_empty() {
				{
					let mut pages = this.pages.lock().unwrap();
					let mut current_page = this.current_page.lock().unwrap();
					let mut pages_history = this.pages_history.lock().unwrap();
					pages_history.push(undone_pages_history.pop().unwrap());
					*pages = pages_history.last().unwrap().clone();
					if *current_page > pages.len() - 1 {
						*current_page = pages.len() - 1;
					}
				}
				this.reload_page_pack();
				this.area.queue_draw();
			}
		}));
		self.application_layout
			.tool_pack
			.pack_start(&redo, false, false, 0);
	}

	/// Pages can be added.
	///
	/// Connects `gtk::Button` to add a page on click.
	fn add_page(&self) {
		let add_page = Button::with_label("+");
		add_page.connect_clicked(clone!(@strong self as this => move |_| {
			let page = Page::new(
				Rc::clone(&this.current_page),
				Rc::clone(&this.image_buffer),
				this.area.clone(),
				&this.application_layout.page_pack,
			);
			let mut pages = this.pages.lock().unwrap();
			pages.push(page);
			this.application_layout.page_pack.show_all();
		}));
		self.application_layout
			.page_pack
			.pack_start(&add_page, false, false, 0);
	}

	/// Pages can be removed.
	///
	/// Connects `gtk::Button` to remove the `self.current_page` on click.
	fn remove_page(&self) {
		let remove_page = Button::with_label("-");
		remove_page.connect_clicked(clone!(@strong self as this => move |_| {
			let mut pages = this.pages.lock().unwrap();
			if pages.len() > 1 {
				let mut current_page = this.current_page.lock().unwrap();
				pages.remove(*current_page);
				*current_page = 0;
				let page_buttons = this.application_layout.page_pack.get_children();
				let last_page_button = &page_buttons[page_buttons.len() - 5];
				this.application_layout.page_pack.remove(last_page_button);
				this.application_layout.page_pack.show_all();
			}
		}));
		self.application_layout
			.page_pack
			.pack_end(&remove_page, false, false, 0);
	}

	/// Pages can be moved.
	///
	/// Connects buttons to move the `self.current_page` up and down.
	fn move_page(&self) {
		let move_up = Button::with_label("↓");
		move_up.connect_clicked(clone!(@strong self as this => move |_| {
			let mut pages = this.pages.lock().unwrap();
			let mut current_page = this.current_page.lock().unwrap();
			if *current_page < pages.len() - 1 {
				let page = pages.remove(*current_page);
				*current_page += 1;
				pages.insert(*current_page, page);
			}
		}));
		self.application_layout
			.page_pack
			.pack_end(&move_up, false, false, 0);

		let move_down = Button::with_label("↑");
		move_down.connect_clicked(clone!(@strong self as this => move |_| {
			let mut pages = this.pages.lock().unwrap();
			let mut current_page = this.current_page.lock().unwrap();
			if *current_page > 0 {
				let page = pages.remove(*current_page);
				*current_page -= 1;
				pages.insert(*current_page, page);
			}
		}));
		self.application_layout
			.page_pack
			.pack_end(&move_down, false, false, 0);
	}

	/// Ensures correct interaction based on `self.drawing_information.current_draw_tool`.
	///
	/// Instantiates all `DrawTool` implementations.
	///
	/// Invokes `self.color_widget()`.
	fn manage_drawing_modes(&self) {
		self.color_widget();

		let pencil = Rc::new(Mutex::new(Pencil::new(
			Rc::clone(&self.drawing_information.current_draw_tool),
			&self.application_layout.tool_pack,
		)));
		let eraser = Rc::new(Mutex::new(Eraser::new(
			Rc::clone(&self.drawing_information.current_draw_tool),
			&self.application_layout.tool_pack,
		)));
		let line_eraser = Rc::new(Mutex::new(LineEraser::new(
			Rc::clone(&self.drawing_information.current_draw_tool),
			&self.application_layout.tool_pack,
		)));
		let line_tool = Rc::new(Mutex::new(LineTool::new(
			Rc::clone(&self.drawing_information.current_draw_tool),
			&self.application_layout.tool_pack,
		)));
		let drag = Rc::new(Mutex::new(Drag::new(
			Rc::clone(&self.drawing_information.current_draw_tool),
			&self.application_layout.tool_pack,
		)));
		let rectangle_selection = Rc::new(Mutex::new(RectangleSelection::new(
			Rc::clone(&self.drawing_information.current_draw_tool),
			&self.application_layout.tool_pack,
			self.area.clone(),
		)));
		let clear = Rc::new(Mutex::new(Clear::new(
			Rc::clone(&self.drawing_information.current_draw_tool),
			&self.application_layout.tool_pack,
		)));

		self.area.connect_motion_notify_event(clone!(@strong self as this => move |_, e| {
			let current_draw_tool = this.drawing_information.current_draw_tool.lock().unwrap();
			let active_draw_tool: Rc<Mutex<dyn DrawTool>> = match *current_draw_tool {
				CurrentDrawTool::Pencil => Rc::clone(&pencil) as _,
				CurrentDrawTool::Eraser => Rc::clone(&eraser) as _,
				CurrentDrawTool::LineEraser => Rc::clone(&line_eraser) as _,
				CurrentDrawTool::LineTool => Rc::clone(&line_tool) as _,
				CurrentDrawTool::Drag => Rc::clone(&drag) as _,
				CurrentDrawTool::RectangleSelection => Rc::clone(&rectangle_selection) as _,
				CurrentDrawTool::Clear => Rc::clone(&clear) as _,
			};
			let rgba = this.drawing_information.rgba.lock().unwrap();
			let pen_size = this.drawing_information.pen_size.lock().unwrap();
			let pen_is_active = this.drawing_information.pen_is_active.lock().unwrap();
			active_draw_tool.lock().unwrap().manipulate(Rc::clone(&this.pages), Rc::clone(&this.current_page),Rc::clone(&this.image_buffer), e.get_position(), *pen_size, *pen_is_active, *rgba);
			this.area.queue_draw();
			Inhibit(false)
		}));
	}

	/// Composes `color_widget` with color selection dialog and predefined colors.
	///
	/// Adds `color_widget` to `self.application_layout.tool_pack`.
	fn color_widget(&self) {
		let color_widget = Box::new(Orientation::Horizontal, 0);

		let predefined_colors = [
			[0.0, 0.0, 0.0, 1.0],
			[1.0, 1.0, 1.0, 1.0],
			[1.0, 0.0, 0.0, 1.0],
			[0.0, 1.0, 0.0, 1.0],
			[0.0, 0.0, 1.0, 1.0],
			[1.0, 1.0, 0.0, 1.0],
			[0.0, 1.0, 1.0, 1.0],
			[1.0, 0.0, 1.0, 1.0],
		];
		for predefined_rgba in predefined_colors.iter() {
			let select_color = Button::new();
			select_color.connect_clicked(
				clone!(@strong self.drawing_information.rgba as rgba, @strong predefined_rgba => move |_| {
					*rgba.lock().unwrap() = predefined_rgba;
				}),
			);
			let predefined_rgba = Some(RGBA {
				red: predefined_rgba[0],
				green: predefined_rgba[1],
				blue: predefined_rgba[2],
				alpha: predefined_rgba[3],
			});
			let color_preview = Label::new(None);
			color_preview.set_size_request(20, 20);
			color_preview.override_background_color(StateFlags::NORMAL, predefined_rgba.as_ref());
			select_color.set_image(Some(&color_preview));
			color_widget.pack_start(&select_color, false, false, 0);
		}

		let color_dialog = Button::with_label("Colors");
		color_dialog.connect_clicked(clone!(@strong self as this => move |_| {
			let rgba = &this.drawing_information.rgba;
			let dialog = Dialog::with_buttons(
				Some("Colors"),
				Some(&this.window.clone()),
				DialogFlags::DESTROY_WITH_PARENT,
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
			content_area.pack_start(&color_preview, false, false, 0);
			for (i, scale) in scales.iter().enumerate() {
				scale.set_value(rgba.lock().unwrap()[i]);
				content_area.add(scale);
					scale.connect_change_value(clone!(@strong color_preview, @strong rgba => move |_, _, v| {
						let mut rgba = rgba.lock().unwrap();
						rgba[i] = v;
						let rgba = Some(RGBA {red: rgba[0], green: rgba[1], blue: rgba[2], alpha: rgba[3]});
						color_preview.override_background_color(StateFlags::NORMAL, rgba.as_ref());
						Inhibit(false)
					}));
			}
			dialog.show_all();
		}));
		color_widget.pack_start(&color_dialog, false, false, 0);

		self.application_layout
			.tool_pack
			.pack_start(&color_widget, false, false, 0);
	}

	/// Displays the position pointer on the canvas in the current color.
	fn position_pointer(&self) {
		self.area.connect_motion_notify_event(
			clone!(@strong self.drawing_information.cursor_position as cursor_position => move |_, e| {
				*cursor_position.lock().unwrap() = Some(e.get_position());
				Inhibit(false)
			}),
		);

		self.area.connect_leave_notify_event(
			clone!(@strong self.drawing_information.cursor_position as cursor_position => move |_, _| {
				*cursor_position.lock().unwrap() = None;
				Inhibit(false)
			}),
		);

		self.area
			.connect_draw(clone!(@strong self as this => move |_, cr| {
				let cursor_position = this.drawing_information.cursor_position.lock().unwrap();
				if cursor_position.is_some() {
					let pen_size = this.drawing_information.pen_size.lock().unwrap();
					let rgba = *this.drawing_information.rgba.lock().unwrap();
					cr.set_source_rgba(
						rgba[0],
						rgba[1],
						rgba[2],
						rgba[3],
					);
					cr.set_line_width(5.0);
					cr.arc(cursor_position.unwrap().0, cursor_position.unwrap().1, *pen_size / 2.0, 0.0, PI * 2.0);
					cr.stroke();
				}
				Inhibit(false)
			}));
	}

	/// Connects a `gtk::FileChooserNative` instance to export the current `Page` to the chosen .png file.
	fn export_png(&self) {
		self.connect_file_dialog(
			FileChooserAction::Save,
			Heap::new(clone!(@strong self as this => move |current_path| {
				this.save_file(&current_path);

				let area_width = this.area.get_allocated_width();
				let area_height = this.area.get_allocated_height();
				let surface = ImageSurface::create(Format::ARgb32, area_width, area_height)
					.expect("Can't create surface.");
				let cr = Context::new(&surface);
				this.context_drawing_mechanics(&cr);

				let mut png = File::create(current_path).expect("Couldn't create file.");
				surface
					.write_to_png(&mut png)
					.expect("Image could not be written out.");
			})),
		);
	}
}
