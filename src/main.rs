use fltk::{app, button::*, draw::*, frame::*, group::*, window::*};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
	let app = app::App::default().with_scheme(app::Scheme::Gtk);

	let mut window = Window::default()
		.with_size(800, 600)
		.center_screen()
		.with_label("Handwritten notetaking editor");

	let mut horizontal_pack = Pack::default().size_of(&window);

	let mut vertical_pack = Pack::default().with_size(150, window.y());

	println!("{:?}", vertical_pack.x());
	for i in 0..3 {
		let button = Button::default()
			.with_size(vertical_pack.x(), 250)
			.with_label("Test");
	}

	vertical_pack.end();
	vertical_pack.set_spacing(10);

	let mut vertical_pack = Pack::default().size_of(&window);

	let mut frame = Frame::default().size_of(&vertical_pack);
	frame.set_color(Color::White);
	frame.set_frame(FrameType::DownBox);

	vertical_pack.end();
	vertical_pack.set_spacing(10);

	horizontal_pack.end();
	horizontal_pack.set_type(PackType::Horizontal);
	horizontal_pack.set_spacing(10);

	window.make_resizable(false);
	window.end();
	window.show();

	app.run().unwrap();
}
