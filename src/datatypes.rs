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

/// Enum representation of possible tools that a user can directly apply to the canvas.
///
/// In every `new()` function of any `DrawTool` the `gtk::Button` is connected on click to set the current_draw_tool to one of the enum values.
/// This is matched to the corresponding DrawTool in the `Application.manage_drawing_modes()` function.
#[derive(PartialEq, Clone, Debug)]
pub enum CurrentDrawTool {
	Pencil,
	Eraser,
	LineEraser,
	LineTool,
	Drag,
	RectangleSelection,
	Clear,
}

/// Trait for a tool that can directly manipulate the canvas.
///
/// All implementations have to get a similarly named enum value in `CurrentDrawTool`.
pub trait DrawTool {
	/// This function is called in every time step that the user interacts with the canvas.
	///
	/// Lines and images can be manipulated depending on the `position` input.
	fn manipulate(
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		_image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		position: (f64, f64),
		pen_size: f64,
		pen_is_active: bool,
		rgba: [f64; 4],
	);

	/// Misc function for finding the `closest_position` in all `lines` of the `current_page`.
	fn closest_line_position(
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		position: (f64, f64),
	) -> Option<(usize, (f64, f64))>
	where
		Self: Sized,
	{
		let mut pages = pages.lock().unwrap();
		let current_page = current_page.lock().unwrap();
		let lines = &mut pages[*current_page].lines;
		let mut lowest_distance = f64::INFINITY;
		let mut line_index = None;
		let mut closest_position = None;
		for (i, line) in lines.iter().enumerate() {
			for point in line.iter() {
				let distance = ((point.position.0 - position.0).powf(2.0)
					+ (point.position.1 - position.1).powf(2.0))
				.sqrt();
				if distance < lowest_distance {
					lowest_distance = distance;
					line_index = Some(i);
					closest_position = Some(point.position);
				};
			}
		}
		let line_index = line_index?;
		let closest_position = closest_position?;
		Some((line_index, closest_position))
	}
}

/// Basic `DrawTool` to create lines.
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
		_image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		position: (f64, f64),
		pen_size: f64,
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
				.push(Drawpoint::new(position, pen_size, rgba));
		}
	}
}

/// Basic `DrawTool` to erase Drawpoints in lines and split where the `Drawpoint` was deleted.
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
		_image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		position: (f64, f64),
		pen_size: f64,
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
					if distance < pen_size {
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

/// Erases the whole `line` on contact with the tool.
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
		_image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		position: (f64, f64),
		pen_size: f64,
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
					if distance < pen_size {
						return false;
					};
				}
				true
			})
		}
	}
}

/// Draws straight lines from the drag `starting_position` to the pointer `position`.
///
/// As many Drawpoints are inserted depending on how long the line is.
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
		_image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		position: (f64, f64),
		pen_size: f64,
		pen_is_active: bool,
		rgba: [f64; 4],
	) {
		if pen_is_active {
			let mut pages = pages.lock().unwrap();
			let current_page = current_page.lock().unwrap();
			let lines = &mut pages[*current_page].lines;
			let starting_point = if lines.last().unwrap().is_empty() {
				Drawpoint::new(position, pen_size, rgba)
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
				lines.push(Drawpoint::new(new_position, pen_size, rgba));
			}
		}
	}
}

/// Enum representation of possible `Drag` tool modes.
#[derive(Clone, Debug)]
pub enum DragMode {
	Line,
	BufferedImage,
	None,
}

/// `Drag` tool for draging the closest line/image on at a time.
///
/// Previous values have to be saved before translating the positions for correct calculations.
#[derive(Clone, Debug)]
pub struct Drag {
	selection_index: usize,
	starting_position: (f64, f64),
	previous_pen_is_active: bool,
	previous_lines: Vec<Vec<Drawpoint>>,
	buffered_image_position: (f64, f64),
	previous_buffered_image_position: (f64, f64),
	mode: DragMode,
}

impl Drag {
	pub fn new(current_draw_tool: Rc<Mutex<CurrentDrawTool>>, pack: &Box) -> Self {
		let button = Button::with_label("Drag");
		let draw_tool = Self {
			selection_index: 0,
			starting_position: (0.0, 0.0),
			previous_pen_is_active: false,
			previous_lines: Vec::<Vec<Drawpoint>>::new(),
			buffered_image_position: (0.0, 0.0),
			previous_buffered_image_position: (0.0, 0.0),
			mode: DragMode::None,
		};
		button.connect_clicked(move |_| {
			*current_draw_tool.lock().unwrap() = CurrentDrawTool::Drag;
		});
		pack.pack_start(&button, false, false, 0);
		draw_tool
	}

	/// Misc function for finding the `closest_position` in all `images` in `current_page`, in this case represented by `image_buffer`.
	pub fn closest_image(
		image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		position: (f64, f64),
	) -> Option<(usize, (f64, f64))> {
		let image_buffer = image_buffer.lock().unwrap();
		let mut lowest_distance = f64::INFINITY;
		let mut buffered_image_index = None;
		let mut closest_point = None;
		for (i, buffered_image) in image_buffer.iter().enumerate() {
			let handle_position = Self::get_handle_position(buffered_image);
			let distance = ((handle_position.0 - position.0).powf(2.0)
				+ (handle_position.1 - position.1).powf(2.0))
			.sqrt();
			if distance < lowest_distance {
				lowest_distance = distance;
				buffered_image_index = Some(i);
				closest_point = Some(handle_position);
			};
		}
		let buffered_image_index = buffered_image_index?;
		let closest_point = closest_point?;
		Some((buffered_image_index, closest_point))
	}

	fn get_handle_position(buffered_image: &BufferedImage) -> (f64, f64) {
		let image = buffered_image.image.lock().unwrap();
		let width = buffered_image.image_surface.get_width() as f64;
		let height = buffered_image.image_surface.get_height() as f64;
		(
			image.position.0 + (width - image.position.0) / 2.0,
			image.position.1 + (height - image.position.1) / 2.0,
		)
	}

	/// Calculates and sets `DragMode` for `self`, depending on pointer `position`.
	fn set_mode(
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		position: (f64, f64),
	) {
		self.mode =
			match Self::closest_line_position(Rc::clone(&pages), Rc::clone(&current_page), position)
			{
				Some((_, closest_line_position)) => {
					match Self::closest_image(Rc::clone(&image_buffer), position) {
						Some((_, closest_buffered_image_position)) => {
							let line_distance = (closest_line_position.0 - position.0).powf(2.0)
								+ (closest_line_position.1 - position.1).powf(2.0);
							let buffered_image_distance =
								(closest_buffered_image_position.0 - position.0).powf(2.0)
									+ (closest_buffered_image_position.1 - position.1).powf(2.0);
							if line_distance <= buffered_image_distance {
								DragMode::Line
							} else {
								DragMode::BufferedImage
							}
						}
						None => DragMode::Line,
					}
				}
				None => match Self::closest_image(Rc::clone(&image_buffer), position) {
					Some(_) => DragMode::BufferedImage,
					None => DragMode::None,
				},
			}
	}

	/// Translates `line` positions depending on drag `vector`.
	fn line_drag(
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		line_index: usize,
		vector: (f64, f64),
	) {
		let mut pages = pages.lock().unwrap();
		let current_page = current_page.lock().unwrap();
		let lines = &mut pages[*current_page].lines;
		if !self.previous_pen_is_active {
			self.previous_lines = lines.clone();
		}
		let line = &mut lines[line_index];
		let previous_line = &mut self.previous_lines[line_index];
		for (point, prev_point) in line.iter_mut().zip(previous_line) {
			point.position.0 = prev_point.position.0 + vector.0;
			point.position.1 = prev_point.position.1 + vector.1;
		}
	}

	/// Translates `image` position depending on drag `vector`.
	fn buffered_image_drag(
		&mut self,
		image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		buffered_image_index: usize,
		closest_buffered_image_position: (f64, f64),
		vector: (f64, f64),
	) {
		let mut image_buffer = image_buffer.lock().unwrap();
		if !self.previous_pen_is_active {
			self.previous_buffered_image_position = closest_buffered_image_position;
		}
		let buffered_image = &mut image_buffer[buffered_image_index];
		let width = buffered_image.image_surface.get_width() as f64;
		let height = buffered_image.image_surface.get_height() as f64;
		let mut image = buffered_image.image.lock().unwrap();
		image.position.0 =
			self.previous_buffered_image_position.0 + vector.0 - (width as f64) / 2.0;
		image.position.1 =
			self.previous_buffered_image_position.1 + vector.1 - (height as f64) / 2.0;
	}
}

impl DrawTool for Drag {
	fn manipulate(
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		position: (f64, f64),
		_pen_size: f64,
		pen_is_active: bool,
		_rgba: [f64; 4],
	) {
		if pen_is_active {
			if !self.previous_pen_is_active {
				self.set_mode(
					Rc::clone(&pages),
					Rc::clone(&current_page),
					Rc::clone(&image_buffer),
					position,
				);
			}
			if !self.previous_pen_is_active {
				self.starting_position = position;
			}
			let vector = (
				position.0 - self.starting_position.0,
				position.1 - self.starting_position.1,
			);
			match self.mode {
				DragMode::Line => {
					if !self.previous_pen_is_active {
						self.selection_index = Self::closest_line_position(
							Rc::clone(&pages),
							Rc::clone(&current_page),
							position,
						)
						.unwrap()
						.0;
					}
					self.line_drag(
						Rc::clone(&pages),
						Rc::clone(&current_page),
						self.selection_index,
						vector,
					);
				}
				DragMode::BufferedImage => {
					let (buffered_image_index, closest_buffered_image_position) =
						Self::closest_image(Rc::clone(&image_buffer), position).unwrap();
					if !self.previous_pen_is_active {
						self.selection_index = buffered_image_index;
					}
					self.buffered_image_drag(
						Rc::clone(&image_buffer),
						self.selection_index,
						closest_buffered_image_position,
						vector,
					)
				}
				DragMode::None => {}
			}
		}
		self.previous_pen_is_active = pen_is_active;
	}
}

/// Enum representation of possible `RectangleSelection` tool modes.
#[derive(Clone, Debug)]
enum RectangleSelectionMode {
	Selection,
	Translation,
}

/// Line elements can be selected by grouping them in a rectangle and then be repositioned.
///
/// Previous values have to be saved before translating the positions for correct calculations.
#[derive(Clone, Debug)]
pub struct RectangleSelection {
	rectangle: Rc<Mutex<[f64; 4]>>,
	previous_rectangle: Rc<Mutex<[f64; 4]>>,
	selection: HashSet<usize>,
	starting_position: (f64, f64),
	previous_pen_is_active: bool,
	previous_lines: Vec<Vec<Drawpoint>>,
	mode: RectangleSelectionMode,
}

impl RectangleSelection {
	pub fn new(
		current_draw_tool: Rc<Mutex<CurrentDrawTool>>,
		pack: &Box,
		area: DrawingArea,
	) -> Self {
		let button = Button::with_label("Rect Selection");
		let draw_tool = Self {
			rectangle: Rc::new(Mutex::new([0.0; 4])),
			previous_rectangle: Rc::new(Mutex::new([0.0; 4])),
			selection: HashSet::<usize>::new(),
			starting_position: (0.0, 0.0),
			previous_pen_is_active: false,
			previous_lines: Vec::<Vec<Drawpoint>>::new(),
			mode: RectangleSelectionMode::Selection,
		};
		let line_matrix = [(0, 1), (2, 1), (2, 3), (0, 3), (0, 1)];
		area.connect_draw(
			clone!(@strong draw_tool as this, @strong current_draw_tool, @strong line_matrix => move |_, cr| {
				let current_draw_tool = current_draw_tool.lock().unwrap();
				if *current_draw_tool == CurrentDrawTool::RectangleSelection {
					let rectangle = this.rectangle.lock().unwrap();
					cr.set_source_rgba(0.0,	0.0, 0.0, 0.5);
					for line in line_matrix.iter() {
						cr.set_line_width(5.0);
						cr.line_to(rectangle[line.0], rectangle[line.1]);
					}
					cr.stroke();
				}
				Inhibit(false)
			}),
		);
		button.connect_clicked(clone!(@strong draw_tool as this => move |_| {
			*current_draw_tool.lock().unwrap() = CurrentDrawTool::RectangleSelection;
			*this.rectangle.lock().unwrap() = [0.0; 4];
		}));
		pack.pack_start(&button, false, false, 0);
		draw_tool
	}

	/// Checks if a `position` is in `self.rectangle`.
	///
	/// Edge cases are excluded.
	fn is_in_rectangle(&self, position: (f64, f64)) -> bool {
		let rectangle = self.rectangle.lock().unwrap();
		position.0 > rectangle[0]
			&& position.1 > rectangle[1]
			&& position.0 < rectangle[2]
			&& position.1 < rectangle[3]
	}

	/// Updates `self.selection` set depending on whether or not one of the `Drawpoint` positions is in `self.rectangle`.
	fn update_selection(&mut self, lines: &Vec<Vec<Drawpoint>>) {
		for (i, line) in lines.iter().enumerate() {
			for point in line.iter() {
				if self.is_in_rectangle(point.position) {
					self.selection.insert(i);
				}
			}
		}
	}

	/// Updates `self.rectangle` depending on pointer `position` and `self.starting_position`.
	fn update_rectangle(&mut self, position: (f64, f64)) {
		let mut rectangle = self.rectangle.lock().unwrap();
		if self.starting_position.0 < position.0 {
			rectangle[0] = self.starting_position.0;
			rectangle[2] = position.0;
		} else {
			rectangle[0] = position.0;
			rectangle[2] = self.starting_position.0;
		}
		if self.starting_position.1 < position.1 {
			rectangle[1] = self.starting_position.1;
			rectangle[3] = position.1;
		} else {
			rectangle[1] = position.1;
			rectangle[3] = self.starting_position.1;
		}
	}

	/// Translates `line` positions depending on the drag `vector`.
	fn translate_positions(&mut self, lines: &mut Vec<Vec<Drawpoint>>, position: (f64, f64)) {
		let mut rectangle = self.rectangle.lock().unwrap();
		let previous_rectangle = self.previous_rectangle.lock().unwrap();
		let vector = (
			position.0 - self.starting_position.0,
			position.1 - self.starting_position.1,
		);
		rectangle[0] = previous_rectangle[0] + vector.0;
		rectangle[1] = previous_rectangle[1] + vector.1;
		rectangle[2] = previous_rectangle[2] + vector.0;
		rectangle[3] = previous_rectangle[3] + vector.1;
		for line_index in self.selection.iter() {
			let line = &mut lines[*line_index];
			let previous_line = &mut self.previous_lines[*line_index];
			for (point, prev_point) in line.iter_mut().zip(previous_line) {
				point.position.0 = prev_point.position.0 + vector.0;
				point.position.1 = prev_point.position.1 + vector.1;
			}
		}
	}

	/// Calculates and sets `RectangleSelectionMode` for `self`, depending on if `self.starting_position` is in `self.rectangle`.
	fn set_mode(&mut self) {
		if self.is_in_rectangle(self.starting_position) {
			self.mode = RectangleSelectionMode::Translation;
		} else {
			self.mode = RectangleSelectionMode::Selection;
		}
	}
}

impl DrawTool for RectangleSelection {
	fn manipulate(
		&mut self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		_image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		position: (f64, f64),
		_pen_size: f64,
		pen_is_active: bool,
		_rgba: [f64; 4],
	) {
		if pen_is_active {
			let mut pages = pages.lock().unwrap();
			let current_page = current_page.lock().unwrap();
			let lines = &mut pages[*current_page].lines;
			if !self.previous_pen_is_active {
				{
					let rectangle = self.rectangle.lock().unwrap();
					let mut previous_rectangle = self.previous_rectangle.lock().unwrap();
					self.starting_position = position;
					self.previous_lines = lines.clone();
					*previous_rectangle = *rectangle;
					self.selection.clear();
				}
				self.set_mode();
				self.update_selection(lines);
			}
			match self.mode {
				RectangleSelectionMode::Translation => self.translate_positions(lines, position),
				RectangleSelectionMode::Selection => self.update_rectangle(position),
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

/// Removes all `lines` on `current_page`.
///
/// Images are excluded.
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
		_image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		_position: (f64, f64),
		_pen_size: f64,
		_pen_is_active: bool,
		_rgba: [f64; 4],
	) {
		let mut pages = pages.lock().unwrap();
		let current_page = current_page.lock().unwrap();
		let lines = &mut pages[*current_page].lines;
		lines.clear();
	}
}

/// Serializable image datatype.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Image {
	pub path: PathBuf,
	pub position: (f64, f64),
}

impl Image {
	pub fn new(path: PathBuf, position: (f64, f64)) -> Self {
		Self { path, position }
	}
}

/// Blueprint for an image instance of type `gtk::ImageSurface`.
#[derive(Clone, Debug)]
pub struct BufferedImage {
	pub image_surface: ImageSurface,
	pub image: Rc<Mutex<Image>>,
}

impl BufferedImage {
	pub fn new(image_surface: ImageSurface, image: Rc<Mutex<Image>>) -> Self {
		Self {
			image_surface,
			image,
		}
	}
}

/// Serializable page datatype that contains all `lines` and `images` of the current page.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Page {
	pub lines: Vec<Vec<Drawpoint>>,
	pub images: Rc<Mutex<Vec<Rc<Mutex<Image>>>>>,
}

impl Page {
	pub fn new(
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		area: DrawingArea,
		pack: &Box,
	) -> Self {
		let page = Self {
			lines: Vec::<Vec<Drawpoint>>::new(),
			images: Rc::new(Mutex::new(Vec::<Rc<Mutex<Image>>>::new())),
		};
		page.connect_pack(pages, current_page, image_buffer, area, pack);
		page
	}

	/// Places a button in the application that shows the page on click.
	///
	/// Connects `self` to a `gtk::Box` with a `gtk::Button` that manipulates `current_page` and `image_buffer`.
	/// When loading the page, `image_buffer` is updated based on the page images.
	pub fn connect_pack(
		&self,
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
		area: DrawingArea,
		pack: &Box,
	) {
		let button = Button::with_label("Page");
		pack.pack_start(&button, false, false, 0);
		button.connect_clicked(
			clone!(@strong self as this, @strong pages, @strong current_page, @strong image_buffer, @strong pack, @strong button => move |_| {
				{
					let button_position = pack.get_child_position(&button);
					let mut current_page = current_page.lock().unwrap();
					*current_page = button_position as usize;
				}
				Self::reload_image_buffer(Rc::clone(&pages), Rc::clone(&current_page), Rc::clone(&image_buffer));
				area.queue_draw();
			}),
		);
		let button_position = pack.get_child_position(&button);
		pack.set_child_position(&button, button_position - 4);
	}

	/// Updates the currently displayed images.
	///
	/// Reloads `self.image_buffer` depending on `self`.
	fn reload_image_buffer(
		pages: Rc<Mutex<Vec<Page>>>,
		current_page: Rc<Mutex<usize>>,
		image_buffer: Rc<Mutex<Vec<BufferedImage>>>,
	) {
		let mut pages = pages.lock().unwrap();
		let current_page = current_page.lock().unwrap();
		let mut image_buffer = image_buffer.lock().unwrap();
		let images = &mut pages[*current_page].images;
		let images = images.lock().unwrap();
		image_buffer.clear();
		for image in images.iter() {
			let image_surface = {
				let image = image.lock().unwrap();
				let mut path = File::open(image.path.clone()).expect("Could not open file.");
				ImageSurface::create_from_png(&mut path).expect("Could not create ImageSurface.")
			};
			let buffered_image = BufferedImage::new(image_surface, Rc::clone(image));
			image_buffer.push(buffered_image);
		}
	}
}

/// Serializable point that can be drawn on the canvas in a `line`.
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
