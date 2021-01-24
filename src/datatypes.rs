use glib::clone;
use gtk::prelude::*;
use gtk::*;
use serde::{Deserialize, Serialize};
use std::boxed::Box as Heap;
use std::rc::Rc;
use std::sync::Mutex;

pub trait Function {
	fn run(&self);
}

#[derive(Clone, Debug)]
pub enum CurrentDrawTool {
	Pencil,
	Eraser,
	LineEraser,
	LineTool,
	Drag,
	Clear,
}

pub trait DrawTool {
	fn manipulate(
		&self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		size: f64,
		rgba: [f64; 4],
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
		rgba: [f64; 4],
	) {
		let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
		lines
			.last_mut()
			.unwrap()
			.push(Drawpoint::new(position, size, rgba));
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
		_rgba: [f64; 4],
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
		_rgba: [f64; 4],
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
		rgba: [f64; 4],
	) {
		let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
		let starting_point = if lines.last().unwrap().is_empty() {
			Drawpoint::new(position, size, rgba)
		} else {
			lines.last().unwrap()[0].clone()
		};
		let lines = lines.last_mut().unwrap();
		let distance = (starting_point.position.0 - position.0).powf(2.0)
			+ (starting_point.position.1 - position.1).powf(2.0);
		let point_count = distance / 10000.0;
		let vector = (
			(position.0 - starting_point.position.0) / point_count,
			(position.1 - starting_point.position.1) / point_count,
		);
		lines.clear();
		lines.push(starting_point.clone());
		for i in 1..point_count as usize {
			let new_position = (
				starting_point.position.0 + vector.0 * (i as f64),
				starting_point.position.1 + vector.1 * (i as f64),
			);
			lines.push(Drawpoint::new(new_position, size, rgba));
		}
	}
}

#[derive(Clone, Debug)]
pub struct Drag {}

impl Drag {
	pub fn new(current_draw_tool: Rc<Mutex<CurrentDrawTool>>, pack: &Box) -> Self {
		let button = Button::with_label("Drag");
		let draw_tool = Self {};
		button.connect_clicked(move |_| {
			*current_draw_tool.lock().unwrap() = CurrentDrawTool::Drag;
		});
		pack.pack_start(&button, false, false, 0);
		draw_tool
	}
}

impl DrawTool for Drag {
	fn manipulate(
		&self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		_size: f64,
		_rgba: [f64; 4],
	) {
		let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
		let mut lowest_distance = f64::INFINITY;
		let mut line_index = None;
		let mut closest_point = None;
		for (i, line) in lines.iter().enumerate() {
			for point in line.iter() {
				let distance = ((point.position.0 - position.0).powf(2.0)
					+ (point.position.1 - position.1).powf(2.0))
				.sqrt();
				if distance < lowest_distance {
					lowest_distance = distance;
					line_index = Some(i);
					closest_point = Some(point.position);
				};
			}
		}
		let closest_point = closest_point.expect("No closest point found.");
		let line_index = line_index.expect("Could not find any elements in lines vector.");
		for mut point in &mut lines[line_index] {
			point.position.0 = point.position.0 - closest_point.0 + position.0;
			point.position.1 = point.position.1 - closest_point.1 + position.1;
		}
	}
}

#[derive(Clone, Debug)]
pub struct Clear {}

impl Clear {
	pub fn new(current_draw_tool: Rc<Mutex<CurrentDrawTool>>, pack: &Box) -> Self {
		let button = Button::with_label("Clear");
		let draw_tool = Self {};
		button.connect_clicked(move |_| {
			*current_draw_tool.lock().unwrap() = CurrentDrawTool::Clear;
		});
		pack.pack_start(&button, false, false, 0);
		draw_tool
	}
}

impl DrawTool for Clear {
	fn manipulate(
		&self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		_position: (f64, f64),
		_size: f64,
		_rgba: [f64; 4],
	) {
		let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
		lines.clear();
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Page {
	pub lines: Vec<Vec<Drawpoint>>,
}

impl Page {
	pub fn new(current_page: Rc<Mutex<usize>>, area: DrawingArea, pack: &Box) -> Self {
		Self::connect_pack(current_page, area, pack);
		Self {
			lines: Vec::<Vec<Drawpoint>>::new(),
		}
	}

	pub fn connect_pack(current_page: Rc<Mutex<usize>>, area: DrawingArea, pack: &Box) {
		let button = Button::with_label("Page");
		pack.pack_start(&button, false, false, 0);
		button.connect_clicked(clone!(@strong pack, @strong button => move |_| {
			let button_position = pack.get_child_position(&button);
			*current_page.lock().unwrap() = button_position as usize;
			area.queue_draw();
		}));
		let button_position = pack.get_child_position(&button);
		pack.set_child_position(&button, button_position - 4);
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Drawpoint {
	pub position: (f64, f64),
	pub line_width: f64,
	pub rgba: [f64; 4],
}

impl Drawpoint {
	pub fn new(position: (f64, f64), line_width: f64, rgba: [f64; 4]) -> Self {
		Self {
			position,
			line_width,
			rgba,
		}
	}
}
