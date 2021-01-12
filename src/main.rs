use fltk::{app, button::*, draw::*, frame::*, group::*, window::*};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
	let app = app::App::default().with_scheme(app::Scheme::Gtk);

	let mut window = Window::default()
		.with_size(800, 600)
		.center_screen()
		.with_label("Handwritten notetaking editor");

	let mut vertical_pack_0 = Pack::default().size_of(&window);

	let mut horizontal_pack_1 = Pack::default().with_size(window.x(), 80);

	for i in 0..5 {
		Button::default()
			.with_size(80, 80)
			.with_label("Toolbar icon");
	}

	horizontal_pack_1.end();
	horizontal_pack_1.set_type(PackType::Horizontal);

	let mut horizontal_pack_0 = Pack::default().size_of(&vertical_pack_0);

	let mut scroll = Scroll::default().with_size(150, window.y());

	let mut vertical_pack_1 = Pack::default().size_of(&scroll);

	for i in 0..3 {
		Button::default()
			.with_size(vertical_pack_1.x(), 250)
			.with_label("Test page");
	}

	vertical_pack_1.end();

	scroll.end();

	let mut vertical_pack_2 = Pack::default().size_of(&window);

	let mut frame = Frame::default().size_of(&vertical_pack_2);
	frame.set_color(Color::White);
	frame.set_frame(FrameType::DownBox);

	vertical_pack_2.end();

	horizontal_pack_0.end();
	horizontal_pack_0.set_type(PackType::Horizontal);

	vertical_pack_0.end();

	window.make_resizable(false);
	window.end();
	window.show();

	app.run().unwrap();
}
