use gtk::prelude::*;
use gtk::*;
use std::borrow::BorrowMut;
use std::boxed::Box as Heap;

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
	number: i32,
	pub preview: Button,
	versions: Vec<Image>,
}

impl Page {
	pub fn new(number: i32) -> Self {
		let button = Button::with_label(&("Page ".to_owned() + &number.to_string()));
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
