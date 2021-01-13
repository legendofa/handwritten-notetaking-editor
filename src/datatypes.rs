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
		Self {
			icon,
			button: Button::with_label(name.unwrap_or("Tool")),
		}
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
		Self {
			number,
			preview: Button::with_label(
				&(std::string::String::from("Page ") + &number.to_string()),
			),
			versions: Vec::new(),
		}
	}
}
