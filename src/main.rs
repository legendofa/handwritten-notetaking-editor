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

	let menu_bar = menu_bar();

	let button_0 = gtk::Button::with_label("Sidebar 0!");
	let button_1 = gtk::Button::with_label("Sidebar 1!");
	let button_2 = gtk::Button::with_label("Draw here!");
	let button_3 = gtk::Button::with_label("Taskbutton 0!");
	let button_4 = gtk::Button::with_label("Taskbutton 1!");
	let button_5 = gtk::Button::with_label("Taskbutton 2!");
	let button_6 = gtk::Button::with_label("Taskbutton 3!");

	let vertical_pack_0 = gtk::Box::new(Orientation::Vertical, 0);
	let vertical_pack_1 = gtk::Box::new(Orientation::Vertical, 0);

	let horizontal_pack_0 = gtk::Box::new(Orientation::Horizontal, 0);
	let horizontal_pack_1 = gtk::Box::new(Orientation::Horizontal, 0);

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

fn menu_bar() -> MenuBar {
	let menu = Menu::new();
	let menu_bar = MenuBar::new();
	let file = MenuItem::with_label("File");
	let file_item = MenuItem::new();
	let file_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
	let file_image = Image::from_file("resources/file.png");
	let file_label = Label::new(Some("File"));
	let folder_item = MenuItem::new();
	let folder_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
	let folder_image = Image::from_icon_name(Some("folder-music-symbolic"), IconSize::Menu);
	let folder_label = Label::new(Some("Folder"));

	file_box.pack_start(&file_image, false, false, 0);
	file_box.pack_start(&file_label, true, true, 0);
	file_item.add(&file_box);
	folder_box.pack_start(&folder_image, false, false, 0);
	folder_box.pack_start(&folder_label, true, true, 0);
	folder_item.add(&folder_box);
	menu.append(&file_item);
	menu.append(&folder_item);
	file.set_submenu(Some(&menu));
	menu_bar.append(&file);

	let other_menu = Menu::new();
	let sub_other_menu = Menu::new();
	let other = MenuItem::with_label("About");
	let sub_other = MenuItem::with_label("This is");
	let sub_other2 = MenuItem::with_label("a prototype");
	let sub_sub_other2 = MenuItem::with_label("Thats");
	let sub_sub_other2_2 = MenuItem::with_label("right");

	sub_other_menu.append(&sub_sub_other2);
	sub_other_menu.append(&sub_sub_other2_2);
	sub_other2.set_submenu(Some(&sub_other_menu));
	other_menu.append(&sub_other);
	other_menu.append(&sub_other2);
	other.set_submenu(Some(&other_menu));
	menu_bar.append(&other);

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
