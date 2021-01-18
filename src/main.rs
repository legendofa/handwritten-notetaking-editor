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

	let vertical_pack_0 = Box::new(Orientation::Vertical, 0);
	let vertical_pack_1 = Box::new(Orientation::Vertical, 0);

	let horizontal_pack_0 = Box::new(Orientation::Horizontal, 0);
	let horizontal_pack_1 = Box::new(Orientation::Horizontal, 0);

	vertical_pack_0.pack_start(&menu_bar, false, false, 0);
	vertical_pack_0.pack_start(&horizontal_pack_0, false, false, 0);
	vertical_pack_0.pack_start(&horizontal_pack_1, true, true, 0);

	horizontal_pack_1.pack_start(&vertical_pack_1, false, false, 0);
	horizontal_pack_1.pack_start(&area, true, true, 0);

	drawing_mechanics(area, &horizontal_pack_0, &vertical_pack_1);

	window.add(&vertical_pack_0);
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

fn drawing_mechanics(area: DrawingArea, pack: &Box, page_pack: &Box) {
	let current_page: Rc<Mutex<usize>> = Rc::new(Mutex::new(0));

	let button_press = Rc::new(Mutex::new(false));

	let drawing_size = Scale::with_range(Orientation::Horizontal, 0.5, 50.0, 0.5);
	pack.pack_start(&drawing_size, true, true, 0);
	let drawing_alpha = Scale::with_range(Orientation::Horizontal, 0.01, 1.0, 0.01);
	pack.pack_start(&drawing_alpha, true, true, 0);

	let pages = vec![Page::new(), Page::new(), Page::new()];
	for (i, page) in pages.iter().enumerate() {
		{
			let current_page = Rc::clone(&current_page);
			page.preview.connect_clicked(move |_| {
				*current_page.lock().unwrap() = i;
			});
		}
		page_pack.pack_start(&page.preview, false, false, 0);
	}

	undo_redo(pack, &area, &pages[*current_page.lock().unwrap()].lines);

	{
		let lines = Rc::clone(&pages[*current_page.lock().unwrap()].lines);
		let button_press = Rc::clone(&button_press);
		let draw_area = area.clone();
		area.connect_motion_notify_event(move |_, e| {
			if *button_press.lock().unwrap() {
				let lines = &mut lines.lock().unwrap();
				lines.last_mut().unwrap().push(Drawpoint::new(
					e.get_position(),
					drawing_size.get_value(),
					(0.0, 0.0, 0.0, drawing_alpha.get_value()),
				));
				draw_area.queue_draw();
			}
			Inhibit(false)
		});
	}
	{
		let lines = Rc::clone(&pages[*current_page.lock().unwrap()].lines);
		let button_press = Rc::clone(&button_press);
		area.connect_button_press_event(move |_, _| {
			*button_press.lock().unwrap() = true;
			lines.lock().unwrap().push(Vec::new());
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
		let lines = Rc::clone(&pages[*current_page.lock().unwrap()].lines);
		area.connect_draw(move |_, cr| {
			cr.set_source_rgb(1.0, 1.0, 1.0);
			cr.paint();
			for stroke in lines.lock().unwrap().iter() {
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
}

fn undo_redo(pack: &Box, area: &DrawingArea, lines: &Rc<Mutex<Vec<Vec<Drawpoint>>>>) {
	let removed_lines = Rc::new(Mutex::new(Vec::<Vec<Drawpoint>>::new()));
	let undo = Button::with_label("Undo");
	{
		let lines = Rc::clone(&lines);
		let removed_lines = Rc::clone(&removed_lines);
		let draw_area = area.clone();
		undo.connect_clicked(move |_| {
			let lines = &mut lines.lock().unwrap();
			let removed_lines = &mut removed_lines.lock().unwrap();
			if !lines.is_empty() {
				removed_lines.push(lines.pop().unwrap());
				draw_area.queue_draw();
			}
		});
	}
	pack.pack_start(&undo, false, false, 0);

	let redo = Button::with_label("Redo");
	{
		let lines = Rc::clone(&lines);
		let removed_lines = Rc::clone(&removed_lines);
		let draw_area = area.clone();
		redo.connect_clicked(move |_| {
			let lines = &mut lines.lock().unwrap();
			let removed_lines = &mut removed_lines.lock().unwrap();
			if !removed_lines.is_empty() {
				lines.push(removed_lines.pop().unwrap());
				draw_area.queue_draw();
			}
		});
	}
	pack.pack_start(&redo, false, false, 0);
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
