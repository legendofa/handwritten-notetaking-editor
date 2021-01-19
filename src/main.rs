pub mod datatypes;
use crate::datatypes::*;
use gdk::*;
use gio::prelude::*;
use glib::*;
use gtk::prelude::*;
use gtk::*;
use std::boxed::Box as Heap;
use std::env::args;
use std::f64::consts::PI;
use std::rc::Rc;
use std::sync::Mutex;

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
	let area = DrawingArea::new();
	area.add_events(EventMask::ALL_EVENTS_MASK);

	let vertical_pack_0 = Box::new(Orientation::Vertical, 0);
	let vertical_pack_1 = Box::new(Orientation::Vertical, 0);

	let horizontal_pack_0 = Box::new(Orientation::Horizontal, 0);
	let horizontal_pack_1 = Box::new(Orientation::Horizontal, 0);

	vertical_pack_0.pack_start(&menu_bar, false, false, 0);
	vertical_pack_0.pack_start(&horizontal_pack_0, false, false, 0);
	vertical_pack_0.pack_start(&horizontal_pack_1, true, true, 0);

	horizontal_pack_1.pack_start(&vertical_pack_1, false, false, 0);
	horizontal_pack_1.pack_start(&area, true, true, 0);

	drawing_mechanics(area, &horizontal_pack_0, &vertical_pack_1);

	window.add(&vertical_pack_0);
}

fn menu_bar(window: &ApplicationWindow) -> MenuBar {
	let menu = Menu::new();
	let menu_bar = MenuBar::new();
	let file = MenuItem::with_label("File");
	let open_file = MenuItem::new();
	open_file.add(&Label::new(Some("Open File")));

	open_file.connect_activate(clone!(@strong window => move |_| {
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

fn drawing_mechanics(area: DrawingArea, pack: &Box, page_pack: &Box) {
	let current_page: Rc<Mutex<usize>> = Rc::new(Mutex::new(0));

	let button_press = Rc::new(Mutex::new(false));

	let pages = Rc::new(Mutex::new(vec![
		Page::new(Rc::clone(&current_page), area.clone(), &page_pack, 0),
		Page::new(Rc::clone(&current_page), area.clone(), &page_pack, 1),
	]));

	add_pages(page_pack, &area, &pages, &current_page);

	undo_redo(pack, &area, &pages, &current_page);

	let size_tool = Scale::with_range(Orientation::Horizontal, 0.5, 50.0, 0.5);
	pack.pack_start(&size_tool, true, true, 0);

	manage_drawing_modes(
		pack,
		&area,
		&pages,
		&current_page,
		&button_press,
		&size_tool,
	);

	{
		let pages = Rc::clone(&pages);
		let current_page = Rc::clone(&current_page);
		let button_press = Rc::clone(&button_press);
		area.connect_button_press_event(move |_, _| {
			let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
			*button_press.lock().unwrap() = true;
			lines.push(Vec::new());
			Inhibit(false)
		});
	}
	{
		let button_press = Rc::clone(&button_press);
		area.connect_button_release_event(move |_, _| {
			*button_press.lock().unwrap() = false;
			Inhibit(false)
		});
	}
	{
		let pages = Rc::clone(&pages);
		let current_page = Rc::clone(&current_page);
		area.connect_draw(move |_, cr| {
			let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
			cr.set_source_rgb(1.0, 1.0, 1.0);
			cr.paint();
			for stroke in lines.iter() {
				for drawpoint in stroke {
					cr.set_source_rgba(
						drawpoint.rgba.0,
						drawpoint.rgba.1,
						drawpoint.rgba.2,
						drawpoint.rgba.3,
					);
					cr.set_line_width(drawpoint.line_width);
					cr.line_to(drawpoint.position.0, drawpoint.position.1);
				}
				cr.stroke();
			}
			Inhibit(false)
		});
	}

	position_pointer(&area, &size_tool);
}

fn undo_redo(
	pack: &Box,
	area: &DrawingArea,
	pages: &Rc<Mutex<Vec<Page>>>,
	current_page: &Rc<Mutex<usize>>,
) {
	let removed_lines = Rc::new(Mutex::new(Vec::<Vec<Drawpoint>>::new()));
	let undo = Button::with_label("Undo");
	{
		let pages = Rc::clone(&pages);
		let current_page = Rc::clone(&current_page);
		let removed_lines = Rc::clone(&removed_lines);
		undo.connect_clicked(clone!(@strong area => move |_| {
			let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
			let removed_lines = &mut removed_lines.lock().unwrap();
			if !lines.is_empty() {
				removed_lines.push(lines.pop().unwrap());
				area.queue_draw();
			}
		}));
	}
	pack.pack_start(&undo, false, false, 0);

	let redo = Button::with_label("Redo");
	{
		let pages = Rc::clone(&pages);
		let current_page = Rc::clone(&current_page);
		let removed_lines = Rc::clone(&removed_lines);
		redo.connect_clicked(clone!(@strong area => move |_| {
			let lines = &mut pages.lock().unwrap()[*current_page.lock().unwrap()].lines;
			let removed_lines = &mut removed_lines.lock().unwrap();
			if !removed_lines.is_empty() {
				lines.push(removed_lines.pop().unwrap());
				area.queue_draw();
			}
		}));
	}
	pack.pack_start(&redo, false, false, 0);
}

fn add_pages(
	page_pack: &Box,
	area: &DrawingArea,
	pages: &Rc<Mutex<Vec<Page>>>,
	current_page: &Rc<Mutex<usize>>,
) {
	let add_page = Button::with_label("+");
	{
		let pages = Rc::clone(&pages);
		let current_page = Rc::clone(&current_page);
		add_page.connect_clicked(clone!(@strong area, @strong page_pack => move |_| {
			let page = Page::new(
				Rc::clone(&current_page),
				area.clone(),
				&page_pack,
				pages.lock().unwrap().len(),
			);
			let pages = &mut pages.lock().unwrap();
			println!("{:?}", page_pack.get_children());
			pages.push(page);
			page_pack.queue_draw();
		}));
	}
	page_pack.pack_start(&add_page, false, false, 0);
}

fn manage_drawing_modes(
	pack: &Box,
	area: &DrawingArea,
	pages: &Rc<Mutex<Vec<Page>>>,
	current_page: &Rc<Mutex<usize>>,
	button_press: &Rc<Mutex<bool>>,
	size_tool: &Scale,
) {
	let drawing_alpha = Scale::with_range(Orientation::Horizontal, 0.01, 1.0, 0.01);
	pack.pack_start(&drawing_alpha, true, true, 0);

	position_pointer(area, &size_tool);

	let current_draw_tool = Rc::new(Mutex::new(CurrentDrawTool::Pencil));
	let pencil = Rc::new(Mutex::new(Heap::new(Pencil::new(
		Rc::clone(&current_draw_tool),
		pack,
	))));
	let eraser = Rc::new(Mutex::new(Heap::new(Eraser::new(
		Rc::clone(&current_draw_tool),
		pack,
	))));
	let line_eraser = Rc::new(Mutex::new(Heap::new(LineEraser::new(
		Rc::clone(&current_draw_tool),
		pack,
	))));
	let line = Rc::new(Mutex::new(Heap::new(LineTool::new(
		Rc::clone(&current_draw_tool),
		pack,
	))));
	let selection = Rc::new(Mutex::new(Heap::new(Selection::new(
		Rc::clone(&current_draw_tool),
		pack,
	))));

	let pages = Rc::clone(&pages);
	let current_page = Rc::clone(&current_page);
	let button_press = Rc::clone(&button_press);
	area.connect_motion_notify_event(clone!(@strong area, @strong size_tool => move |_, e| {
		if *button_press.lock().unwrap() {
			let active_draw_tool: Heap<dyn DrawTool> = match *current_draw_tool.lock().unwrap() {
				CurrentDrawTool::Pencil => Heap::new(*Rc::clone(&pencil).lock().unwrap().clone()),
				CurrentDrawTool::Eraser => Heap::new(*Rc::clone(&eraser).lock().unwrap().clone()),
				CurrentDrawTool::LineEraser => Heap::new(*Rc::clone(&line_eraser).lock().unwrap().clone()),
				CurrentDrawTool::LineTool => Heap::new(*Rc::clone(&line).lock().unwrap().clone()),
				CurrentDrawTool::Selection => Heap::new(*Rc::clone(&selection).lock().unwrap().clone()),
			};
			active_draw_tool.manipulate(Rc::clone(&pages), Rc::clone(&current_page), e.get_position(), size_tool.get_value(), drawing_alpha.get_value());
		}
		area.queue_draw();
		Inhibit(false)
	}));
}

fn position_pointer(area: &DrawingArea, size_tool: &Scale) {
	let cursor_position = Rc::new(Mutex::new(Some((0.0, 0.0))));

	{
		let cursor_position = Rc::clone(&cursor_position);
		area.connect_motion_notify_event(clone!(@strong area => move |_, e| {
			*cursor_position.lock().unwrap() = Some(e.get_position());
			Inhibit(false)
		}));
	}
	{
		let cursor_position = Rc::clone(&cursor_position);
		area.connect_leave_notify_event(move |_, _| {
			*cursor_position.lock().unwrap() = None;
			Inhibit(false)
		});
	}
	{
		let cursor_position = Rc::clone(&cursor_position);
		area.connect_draw(clone!(@strong size_tool => move |_, cr| {
			let cursor_position = *cursor_position.lock().unwrap();
			if cursor_position.is_some() {
				cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
				cr.set_line_width(5.0);
				cr.arc(cursor_position.unwrap().0, cursor_position.unwrap().1, size_tool.get_value() / 2.0, 0.0, PI * 2.0);
				cr.stroke();
			}
			Inhibit(false)
		}));
	}
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
