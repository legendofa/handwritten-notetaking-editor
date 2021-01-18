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

	open_file.connect_activate(clone!(@strong window => move |_| {
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

	let pages = Rc::new(Mutex::new(vec![Page::new(), Page::new()]));
	for (i, page) in pages.lock().unwrap().iter().enumerate() {
		page.connect(Rc::clone(&current_page), area.clone(), i);
		page_pack.pack_start(&page.preview, false, false, 0);
	}

	add_pages(page_pack, &area, &pages, &current_page);

	undo_redo(pack, &area, &pages, &current_page);

	{
		let pages = Rc::clone(&pages);
		let current_page = Rc::clone(&current_page);
		let button_press = Rc::clone(&button_press);
		area.connect_motion_notify_event(clone!(@strong area => move |_, e| {
			if *button_press.lock().unwrap() {
				let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
				lines.last_mut().unwrap().push(Drawpoint::new(
					e.get_position(),
					drawing_size.get_value(),
					(0.0, 0.0, 0.0, drawing_alpha.get_value()),
				));
				area.queue_draw();
			}
			Inhibit(false)
		}));
	}
	{
		let pages = Rc::clone(&pages);
		let current_page = Rc::clone(&current_page);
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
		let pages = Rc::clone(&pages);
		let current_page = Rc::clone(&current_page);
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
}

fn undo_redo(
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
			let page = Page::new();
			page.connect(
				Rc::clone(&current_page),
				area.clone(),
				pages.lock().unwrap().len(),
			);
			let pages = &mut pages.lock().unwrap();
			page_pack.pack_start(&page.preview, false, false, 0);
			println!("{:?}", page_pack.get_children());
			pages.push(page);
			page_pack.queue_draw();
		}));
	}
	page_pack.pack_start(&add_page, false, false, 0);
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
