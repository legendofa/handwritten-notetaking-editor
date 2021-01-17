pub mod datatypes;
use crate::datatypes::*;
use gdk::*;
use gio::prelude::*;
use glib::*;
use gtk::prelude::*;
use gtk::*;
use std::boxed::Box as Heap;
use std::env::args;
use std::rc::Rc;
use std::sync::Mutex;
use std::{thread, time};

fn build_ui(application: &Application) {
	let window = gtk::ApplicationWindow::new(application);

	window.set_title("Handwritten notetaking editor");
	window.set_border_width(10);
	window.set_position(WindowPosition::Center);
	window.set_default_size(800, 600);

	application_layout(&window);

	window.show_all();
}

fn application_layout(window: &ApplicationWindow) {
	let menu_bar = menu_bar(&window);
	let area = DrawingArea::new();
	area.add_events(EventMask::ALL_EVENTS_MASK);
	area.add_events(EventMask::ALL_EVENTS_MASK);

	let vertical_pack_0 = Box::new(Orientation::Vertical, 0);
	let vertical_pack_1 = Box::new(Orientation::Vertical, 0);

	let horizontal_pack_0 = Box::new(Orientation::Horizontal, 0);
	let horizontal_pack_1 = Box::new(Orientation::Horizontal, 0);

	vertical_pack_0.pack_start(&menu_bar, false, false, 0);
	vertical_pack_0.pack_start(&horizontal_pack_0, false, false, 0);
	vertical_pack_0.pack_start(&horizontal_pack_1, true, true, 0);

	for tool in tool_items() {
		horizontal_pack_0.pack_start(&tool.button, false, false, 0);
	}

	horizontal_pack_1.pack_start(&vertical_pack_1, false, false, 0);
	horizontal_pack_1.pack_start(&area, true, true, 0);

	for page in pages() {
		vertical_pack_1.pack_start(&page.preview, false, false, 0);
	}

	drawing_mechanics(area);

	window.add(&vertical_pack_0);
}

fn pages() -> Vec<Page> {
	vec![
		Page::new(1),
		Page::new(2),
		Page::new(3),
		Page::new(4),
		Page::new(5),
	]
}

fn tool_items() -> Vec<Tool> {
	vec![
		Tool::new(
			Some("Undo"),
			None,
			Heap::new(Messenger::new("Undo an action...")),
		),
		Tool::new(
			Some("Redo"),
			None,
			Heap::new(Messenger::new("Redo an action...")),
		),
		Tool::new(
			Some("Paintbrush"),
			None,
			Heap::new(Messenger::new("Use the Paintbrush...")),
		),
		Tool::new(
			Some("Eraser"),
			None,
			Heap::new(Messenger::new("Use the eraser...")),
		),
	]
}

fn menu_bar(window: &ApplicationWindow) -> MenuBar {
	let menu = Menu::new();
	let menu_bar = MenuBar::new();
	let file = MenuItem::with_label("File");
	let open_file = MenuItem::new();
	open_file.add(&Label::new(Some("Open File")));

	open_file.connect_activate(clone!(@weak window => move |_| {
		println!("File should be openend here...");
	}));

	let save_file = MenuItem::new();
	save_file.add(&Label::new(Some("Save File")));

	save_file.connect_activate(|_| {
		println!("File should be saved here...");
	});

	menu.append(&open_file);
	menu.append(&save_file);
	file.set_submenu(Some(&menu));
	menu_bar.append(&file);

	menu_bar
}

fn drawing_mechanics(area: DrawingArea) {
	let positions = Rc::new(Mutex::new(Vec::<(f64, f64)>::new()));
	let button_press = Rc::new(Mutex::new(false));

	{
		let positions = Rc::clone(&positions);
		let button_press = Rc::clone(&button_press);
		let draw_area = area.clone();
		area.connect_motion_notify_event(move |_, e| {
			if *button_press.lock().unwrap() {
				let positions = &mut positions.lock().unwrap();
				positions.push(e.get_position());
				draw_area.queue_draw();
			}
			Inhibit(false)
		});
	}

	{
		let button_press = Rc::clone(&button_press);
		area.connect_button_press_event(move |_, _| {
			*button_press.lock().unwrap() = true;
			Inhibit(false)
		});
	}

	{
		let positions = Rc::clone(&positions);
		let button_press = Rc::clone(&button_press);
		area.connect_button_release_event(move |_, _| {
			*button_press.lock().unwrap() = false;
			Inhibit(false)
		});
	}

	{
		let positions = Rc::clone(&positions);
		area.connect_draw(move |_, cr| {
			// paint canvas white
			cr.set_source_rgb(1.0, 1.0, 1.0);
			cr.paint();
			// draw 100 random black lines
			cr.set_source_rgb(0.0, 0.0, 0.0);
			for point in positions.lock().unwrap().iter() {
				cr.line_to(point.0, point.1);
			}
			cr.stroke();
			Inhibit(false)
		});
	}
}

fn main() {
	let application =
		gtk::Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
			.expect("Initialization failed...");

	application.connect_activate(|app| {
		build_ui(app);
	});

	application.run(&args().collect::<Vec<_>>());
}
