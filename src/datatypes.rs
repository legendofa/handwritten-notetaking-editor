use cairo::ImageSurface;
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;
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
	RectangleSelection,
	Clear,
}

pub trait DrawTool {
	fn manipulate(
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		size: f64,
		pen_is_active: bool,
		rgba: [f64; 4],
	);

	fn closest_point(
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
	) -> (usize, (f64, f64))
	where
		Self: Sized,
	{
		let mut pages = pages.lock().unwrap();
		let current_page = current_page.lock().unwrap();
		let lines = &mut pages[*current_page].lines;
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
		let line_index = line_index.expect("Could not find any elements in lines vector.");
		let closest_point = closest_point.expect("No closest point found.");
		(line_index, closest_point)
	}
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
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		size: f64,
		pen_is_active: bool,
		rgba: [f64; 4],
	) {
		if pen_is_active {
			let mut pages = pages.lock().unwrap();
			let current_page = current_page.lock().unwrap();
			let lines = &mut pages[*current_page].lines;
			lines
				.last_mut()
				.unwrap()
				.push(Drawpoint::new(position, size, rgba));
		}
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
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		size: f64,
		pen_is_active: bool,
		_rgba: [f64; 4],
	) {
		if pen_is_active {
			let mut pages = pages.lock().unwrap();
			let current_page = current_page.lock().unwrap();
			let lines = &mut pages[*current_page].lines;
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
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		size: f64,
		pen_is_active: bool,
		_rgba: [f64; 4],
	) {
		if pen_is_active {
			let mut pages = pages.lock().unwrap();
			let current_page = current_page.lock().unwrap();
			let lines = &mut pages[*current_page].lines;
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
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		size: f64,
		pen_is_active: bool,
		rgba: [f64; 4],
	) {
		if pen_is_active {
			let mut pages = pages.lock().unwrap();
			let current_page = current_page.lock().unwrap();
			let lines = &mut pages[*current_page].lines;
			let starting_point = if lines.last().unwrap().is_empty() {
				Drawpoint::new(position, size, rgba)
			} else {
				lines.last().unwrap()[0].clone()
			};
			let distance = (starting_point.position.0 - position.0).powf(2.0)
				+ (starting_point.position.1 - position.1).powf(2.0);
			let point_count = distance / 10000.0;
			let vector = (
				(position.0 - starting_point.position.0) / point_count,
				(position.1 - starting_point.position.1) / point_count,
			);
			let lines = lines.last_mut().unwrap();
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
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		_size: f64,
		pen_is_active: bool,
		_rgba: [f64; 4],
	) {
		if pen_is_active {
			let (line_index, closest_point) =
				Self::closest_point(Rc::clone(&pages), Rc::clone(&current_page), position);
			let mut pages = pages.lock().unwrap();
			let current_page = current_page.lock().unwrap();
			let lines = &mut pages[*current_page].lines;
			for mut point in &mut lines[line_index] {
				point.position.0 = point.position.0 - closest_point.0 + position.0;
				point.position.1 = point.position.1 - closest_point.1 + position.1;
			}
		}
	}
}

#[derive(Clone, Debug)]
pub struct RectangleSelection {
	rectangle: [f64; 4],
	selection: HashSet<usize>,
	starting_position: (f64, f64),
	previous_pen_is_active: bool,
	previous_lines: Vec<Vec<Drawpoint>>,
}

impl RectangleSelection {
	pub fn new(current_draw_tool: Rc<Mutex<CurrentDrawTool>>, pack: &Box) -> Self {
		let button = Button::with_label("Rect Selection");
		let draw_tool = Self {
			rectangle: [0.0; 4],
			selection: HashSet::<usize>::new(),
			starting_position: (0.0, 0.0),
			previous_pen_is_active: false,
			previous_lines: Vec::<Vec<Drawpoint>>::new(),
		};
		button.connect_clicked(move |_| {
			*current_draw_tool.lock().unwrap() = CurrentDrawTool::RectangleSelection;
		});
		pack.pack_start(&button, false, false, 0);
		draw_tool
	}

	fn is_in_rectangle(&self, position: (f64, f64)) -> bool {
		position.0 > self.rectangle[0]
			&& position.1 > self.rectangle[1]
			&& position.0 < self.rectangle[2]
			&& position.1 < self.rectangle[3]
	}

	fn draw_rectangle(&self, lines: &mut Vec<Vec<Drawpoint>>) {
		let line_matrix = [(0, 1), (2, 1), (2, 3), (0, 3), (0, 1)];
		let lines = lines.last_mut().unwrap();
		lines.clear();
		for line in line_matrix.iter() {
			let new_position = (self.rectangle[line.0], self.rectangle[line.1]);
			lines.push(Drawpoint::new(new_position, 5.0, [0.0, 0.0, 0.0, 1.0]));
		}
	}

	fn update_selection(&mut self, lines: &Vec<Vec<Drawpoint>>) {
		for (i, line) in lines.iter().enumerate() {
			for point in line.iter() {
				if self.is_in_rectangle(point.position) {
					self.selection.insert(i);
				}
			}
		}
	}
}

impl DrawTool for RectangleSelection {
	fn manipulate(
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
		_size: f64,
		pen_is_active: bool,
		_rgba: [f64; 4],
	) {
		if pen_is_active {
			let mut pages = pages.lock().unwrap();
			let current_page = current_page.lock().unwrap();
			let lines = &mut pages[*current_page].lines;
			if !self.previous_pen_is_active {
				self.starting_position = position;
				self.selection.clear();
				self.update_selection(&lines);
				self.previous_lines = lines.clone();
			}
			if self.is_in_rectangle(position) {
				let vector = (
					position.0 - self.starting_position.0,
					position.1 - self.starting_position.1,
				);
				for line_index in self.selection.iter() {
					let line = &mut lines[*line_index];
					let previous_line = &mut self.previous_lines[*line_index];
					for (point, prev_point) in line.iter_mut().zip(previous_line) {
						point.position.0 = prev_point.position.0 + vector.0;
						point.position.1 = prev_point.position.1 + vector.1;
					}
				}
			} else {
				self.rectangle = [
					self.starting_position.0,
					self.starting_position.1,
					position.0,
					position.1,
				];
				self.draw_rectangle(lines);
			}
		} else {
			if self.previous_pen_is_active {
				let mut pages = pages.lock().unwrap();
				let current_page = current_page.lock().unwrap();
				let lines = &mut pages[*current_page].lines;
				let lines = lines.last_mut().unwrap();
				lines.clear();
			}
		}
		self.previous_pen_is_active = pen_is_active;
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
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		_position: (f64, f64),
		_size: f64,
		_pen_is_active: bool,
		_rgba: [f64; 4],
	) {
		let mut pages = pages.lock().unwrap();
		let current_page = current_page.lock().unwrap();
		let lines = &mut pages[*current_page].lines;
		lines.clear();
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Page {
	pub lines: Vec<Vec<Drawpoint>>,
	pub images: Rc<Mutex<Vec<PathBuf>>>,
}

impl Page {
	pub fn new(
		current_page: Rc<Mutex<usize>>,
		image_buffer: Rc<Mutex<Vec<ImageSurface>>>,
		area: DrawingArea,
		pack: &Box,
	) -> Self {
		let page = Self {
			lines: Vec::<Vec<Drawpoint>>::new(),
			images: Rc::new(Mutex::new(Vec::<PathBuf>::new())),
		};
		page.connect_pack(current_page, image_buffer, area, pack);
		page
	}

	pub fn connect_pack(
		&self,
		current_page: Rc<Mutex<usize>>,
		image_buffer: Rc<Mutex<Vec<ImageSurface>>>,
		area: DrawingArea,
		pack: &Box,
	) {
		let button = Button::with_label("Page");
		pack.pack_start(&button, false, false, 0);
		button.connect_clicked(
			clone!(@strong self as this, @strong image_buffer, @strong pack, @strong button => move |_| {
				let button_position = pack.get_child_position(&button);
				let mut current_page = current_page.lock().unwrap();
				let images = this.images.lock().unwrap();
				let mut image_buffer = image_buffer.lock().unwrap();
				image_buffer.clear();
				for image in images.iter() {
					let mut image = File::open(image).expect("Could not open file.");
					let image_surface =
						ImageSurface::create_from_png(&mut image).expect("Could not create ImageSurface.");
					image_buffer.push(image_surface);
				}
				*current_page = button_position as usize;
				area.queue_draw();
			}),
		);
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
