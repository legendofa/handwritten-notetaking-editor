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

#[derive(Clone, Debug)]
pub struct Page {
	pub preview: Button,
	pub lines: Vec<Vec<Drawpoint>>,
}

impl Page {
	pub fn new(
		current_page: Rc<Mutex<usize>>,
		draw_area: DrawingArea,
		pack: &Box,
		number: usize,
	) -> Self {
		let button = Button::with_label("Page");
		button.connect_clicked(move |_| {
			*current_page.lock().unwrap() = number;
			draw_area.queue_draw();
		});
		pack.pack_start(&button, false, false, 0);
		Self {
			preview: button,
			lines: Vec::<Vec<Drawpoint>>::new(),
		}
	}
}

#[derive(Clone, Debug)]
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
