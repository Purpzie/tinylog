#[macro_use]
extern crate log;
use std::env;

fn main() {
	env::set_var("RUST_LOG", "trace");
	tinylog::init();
	error!("test");
	warn!("test");
	info!("test");
	debug!("test");
	trace!("test");
}
