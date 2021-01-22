pub mod application;
pub mod datatypes;
use crate::application::Application;
use gio::prelude::*;
use std::env::args;

fn main() {
	let application =
		gtk::Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
			.expect("Initialization failed...");

	application.connect_activate(move |app| {
		Application::new().build_ui(app);
	});

	application.run(&args().collect::<Vec<_>>());
}
