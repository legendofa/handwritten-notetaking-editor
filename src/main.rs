use gio::prelude::*;
use glib::*;
use gtk::prelude::*;
use gtk::*;

use std::env::args;

fn build_ui(application: &gtk::Application) {
	let window = gtk::ApplicationWindow::new(application);

	window.set_title("Handwritten notetaking editor");
	window.set_border_width(10);
	window.set_position(gtk::WindowPosition::Center);
	window.set_default_size(800, 600);

	let menu_bar = menu_bar(&window);

	let button_0 = Button::with_label("Sidebar 0!");

	button_0.connect_clicked(|_| {
		println!("Hello");
	});

	let button_1 = Button::with_label("Sidebar 1!");
	let button_2 = DrawingArea::new();
	let button_3 = Button::with_label("Taskbutton 0!");
	let button_4 = Button::with_label("Taskbutton 1!");
	let button_5 = Button::with_label("Taskbutton 2!");
	let button_6 = Button::with_label("Taskbutton 3!");

	let vertical_pack_0 = Box::new(Orientation::Vertical, 0);
	let vertical_pack_1 = Box::new(Orientation::Vertical, 0);

	let horizontal_pack_0 = Box::new(Orientation::Horizontal, 0);
	let horizontal_pack_1 = Box::new(Orientation::Horizontal, 0);

	vertical_pack_0.pack_start(&menu_bar, false, false, 0);
	vertical_pack_0.pack_start(&horizontal_pack_0, false, false, 0);
	vertical_pack_0.pack_start(&horizontal_pack_1, true, true, 0);

	horizontal_pack_0.pack_start(&button_3, false, false, 0);
	horizontal_pack_0.pack_start(&button_4, false, false, 0);
	horizontal_pack_0.pack_start(&button_5, false, false, 0);
	horizontal_pack_0.pack_start(&button_6, false, false, 0);

	horizontal_pack_1.pack_start(&vertical_pack_1, false, false, 0);
	horizontal_pack_1.pack_start(&button_2, true, true, 0);

	vertical_pack_1.pack_start(&button_0, false, false, 0);
	vertical_pack_1.pack_start(&button_1, false, false, 0);

	window.add(&vertical_pack_0);
	window.show_all();
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

fn main() {
	let application =
		gtk::Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
			.expect("Initialization failed...");

	application.connect_activate(|app| {
		build_ui(app);
	});

	application.run(&args().collect::<Vec<_>>());
}
