use gio::prelude::*;
use glib::*;
use gtk::prelude::*;
use gtk::*;

pub trait Tool {
	fn apply();
}

pub struct DemoTool {
	icon: Option<Image>,
	pub button: Button,
}

impl DemoTool {
	pub fn new(name: Option<&str>, icon: Option<Image>) -> Self {
		let button = Button::with_label(name.unwrap_or("Tool"));
		button.connect_clicked(|_| {
			Self::apply();
		});
		Self { icon, button }
	}
}

impl Tool for DemoTool {
	fn apply() {
		println!("Use this tool...");
	}
}

pub struct Page {
	number: i32,
	pub preview: Button,
	versions: Vec<Image>,
}

impl Page {
	pub fn new(number: i32) -> Self {
		let button =
			Button::with_label(&(std::string::String::from("Page ") + &number.to_string()));
		button.connect_clicked(|_| {
			Self::display();
		});
		Self {
			number,
			preview: button,
			versions: Vec::new(),
		}
	}

	fn display() {
		println!("Display this page...");
	}
}
