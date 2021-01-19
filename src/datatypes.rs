use gtk::prelude::*;
use gtk::*;
use std::boxed::Box as Heap;
use std::rc::Rc;
use std::sync::Mutex;

pub trait Function {
	fn run(&self);
}

pub enum CurrentDrawTool {
	Pencil,
	Eraser,
	LineEraser,
	LineTool,
	Selection,
}

pub trait DrawTool {
	fn manipulate(
		&self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		size: f64,
		alpha: f64,
	);
}

#[derive(Clone, Debug)]
pub struct Pencil {}

impl Pencil {
	pub fn new(current_draw_tool: Rc<Mutex<CurrentDrawTool>>, pack: &Box) -> Self {
		let button = Button::with_label("Pen");
		let draw_tool = Self {};
		button.connect_clicked(move |_| {
			*current_draw_tool.lock().unwrap() = CurrentDrawTool::Pencil;
		});
		pack.pack_start(&button, false, false, 0);
		draw_tool
	}
}

impl DrawTool for Pencil {
	fn manipulate(
		&self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		size: f64,
		alpha: f64,
	) {
		let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
		lines
			.last_mut()
			.unwrap()
			.push(Drawpoint::new(position, size, (0.0, 0.0, 0.0, alpha)));
	}
}

#[derive(Clone, Debug)]
pub struct Eraser {}

impl Eraser {
	pub fn new(current_draw_tool: Rc<Mutex<CurrentDrawTool>>, pack: &Box) -> Self {
		let button = Button::with_label("Eraser");
		let draw_tool = Self {};
		button.connect_clicked(move |_| {
			*current_draw_tool.lock().unwrap() = CurrentDrawTool::Eraser;
		});
		pack.pack_start(&button, false, false, 0);
		draw_tool
	}
}

impl DrawTool for Eraser {
	fn manipulate(
		&self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		size: f64,
		alpha: f64,
	) {
		let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
		let mut removal_queue: Vec<(usize, usize)> = Vec::new();
		for (i, line) in lines.iter().enumerate() {
			for (j, point) in line.iter().enumerate() {
				let distance = ((point.position.0 - position.0).powf(2.0)
					+ (point.position.1 - position.1).powf(2.0))
				.sqrt();
				if distance < size {
					removal_queue.push((i, j));
				};
			}
		}
		let mut new_element_count = 0;
		for indices in removal_queue {
			let i = indices.0 + new_element_count;
			let j = indices.1;
			if i < lines.len() {
				if j < lines[i].len() {
					let line = lines[i].split_off(j);
					lines.insert(i + 1, line);
					new_element_count += 1;
				}
			}
		}
	}
}

#[derive(Clone, Debug)]
pub struct LineEraser {}

impl LineEraser {
	pub fn new(current_draw_tool: Rc<Mutex<CurrentDrawTool>>, pack: &Box) -> Self {
		let button = Button::with_label("Line Eraser");
		let draw_tool = Self {};
		button.connect_clicked(move |_| {
			*current_draw_tool.lock().unwrap() = CurrentDrawTool::LineEraser;
		});
		pack.pack_start(&button, false, false, 0);
		draw_tool
	}
}

impl DrawTool for LineEraser {
	fn manipulate(
		&self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		size: f64,
		alpha: f64,
	) {
		let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
		lines.retain(|line| {
			for point in line {
				let distance = ((point.position.0 - position.0).powf(2.0)
					+ (point.position.1 - position.1).powf(2.0))
				.sqrt();
				if distance < size {
					return false;
				};
			}
			true
		})
	}
}

#[derive(Clone, Debug)]
pub struct LineTool {}

impl LineTool {
	pub fn new(current_draw_tool: Rc<Mutex<CurrentDrawTool>>, pack: &Box) -> Self {
		let button = Button::with_label("Line Tool");
		let draw_tool = Self {};
		button.connect_clicked(move |_| {
			*current_draw_tool.lock().unwrap() = CurrentDrawTool::LineTool;
		});
		pack.pack_start(&button, false, false, 0);
		draw_tool
	}
}

impl DrawTool for LineTool {
	fn manipulate(
		&self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		size: f64,
		alpha: f64,
	) {
		let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
		let starting_point = if lines.last().unwrap().is_empty() {
			Drawpoint::new(position, size, (0.0, 0.0, 0.0, alpha))
		} else {
			lines.last().unwrap()[0].clone()
		};
		let lines = lines.last_mut().unwrap();
		lines.clear();
		lines.push(starting_point);
		lines.push(Drawpoint::new(position, size, (0.0, 0.0, 0.0, alpha)));
	}
}

#[derive(Clone, Debug)]
pub struct Selection {}

impl Selection {
	pub fn new(current_draw_tool: Rc<Mutex<CurrentDrawTool>>, pack: &Box) -> Self {
		let button = Button::with_label("Selection");
		let draw_tool = Self {};
		button.connect_clicked(move |_| {
			*current_draw_tool.lock().unwrap() = CurrentDrawTool::Selection;
		});
		pack.pack_start(&button, false, false, 0);
		draw_tool
	}
}

impl DrawTool for Selection {
	fn manipulate(
		&self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		size: f64,
		alpha: f64,
	) {
		let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
		lines.clear();
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
		area: DrawingArea,
		pack: &Box,
		number: usize,
	) -> Self {
		let button = Button::with_label("Page");
		button.connect_clicked(move |_| {
			*current_page.lock().unwrap() = number;
			area.queue_draw();
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
