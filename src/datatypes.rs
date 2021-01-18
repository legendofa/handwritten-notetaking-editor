use gtk::prelude::*;
use gtk::*;
use std::boxed::Box as Heap;
use std::rc::Rc;
use std::sync::Mutex;

pub trait Function {
	fn run(&self);
}

pub struct Messenger<'a> {
	message: &'a str,
}

impl Messenger<'_> {
	pub fn new(message: &'static str) -> Self {
		Self { message }
	}
}

impl Function for Messenger<'_> {
	fn run(&self) {
		println!("{}", self.message);
	}
}

pub struct Tool {
	icon: Option<Image>,
	pub button: Button,
}

impl Tool {
	pub fn new(name: Option<&str>, icon: Option<Image>, function: Heap<dyn Function>) -> Self {
		let button = Button::with_label(name.unwrap_or("Tool"));
		button.connect_clicked(move |_| function.run());
		Self { icon, button }
	}
}

pub struct Page {
	pub preview: Button,
	pub lines: Rc<Mutex<Vec<Vec<Drawpoint>>>>,
}

impl Page {
	pub fn new() -> Self {
		let button = Button::with_label("Page");
		Self {
			preview: button,
			lines: Rc::new(Mutex::new(Vec::<Vec<Drawpoint>>::new())),
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct Drawpoint {
	pub position: (f64, f64),
	pub line_width: f64,
	pub rgba: (f64, f64, f64, f64),
}

impl Drawpoint {
	pub fn new(position: (f64, f64), line_width: f64, rgba: (f64, f64, f64, f64)) -> Self {
		Self {
			position,
			line_width,
			rgba,
		}
	}
}
