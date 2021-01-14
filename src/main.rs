pub mod datatypes;
use crate::datatypes::*;
use gio::prelude::*;
use glib::*;
use gtk::prelude::*;
use gtk::*;
use std::env::args;

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
	let button_2 = DrawingArea::new();

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
	horizontal_pack_1.pack_start(&button_2, true, true, 0);

	for page in pages() {
		vertical_pack_1.pack_start(&page.preview, false, false, 0);
	}

	drawing_mechanics(button_2);

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

fn tool_items() -> Vec<DemoTool> {
	vec![
		DemoTool::new(Some("Undo"), None),
		DemoTool::new(Some("Redo"), None),
		DemoTool::new(Some("Paintbrush"), None),
		DemoTool::new(Some("Eraser"), None),
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
	area.connect_draw(|_, cr| {
		cr.set_dash(&[3., 2., 1.], 1.);

		cr.scale(500f64, 500f64);

		cr.set_source_rgb(250.0 / 255.0, 224.0 / 255.0, 55.0 / 255.0);
		cr.paint();

		cr.set_line_width(0.05);

		// border
		cr.set_source_rgb(0.3, 0.3, 0.3);
		cr.rectangle(0.0, 0.0, 1.0, 1.0);
		cr.stroke();

		cr.set_line_width(0.03);

		// draw circle
		cr.arc(0.5, 0.5, 0.4, 0.0, 3.14 * 2.);
		cr.stroke();

		// mouth
		let mouth_top = 0.68;
		let mouth_width = 0.38;

		let mouth_dx = 0.10;
		let mouth_dy = 0.10;

		cr.move_to(0.50 - mouth_width / 2.0, mouth_top);
		cr.curve_to(
			0.50 - mouth_dx,
			mouth_top + mouth_dy,
			0.50 + mouth_dx,
			mouth_top + mouth_dy,
			0.50 + mouth_width / 2.0,
			mouth_top,
		);

		cr.stroke();

		let eye_y = 0.38;
		let eye_dx = 0.15;
		cr.arc(0.5 - eye_dx, eye_y, 0.05, 0.0, 3.14 * 2.);
		cr.fill();

		cr.arc(0.5 + eye_dx, eye_y, 0.05, 0.0, 3.14 * 2.);
		cr.fill();

		Inhibit(false)
	});
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
