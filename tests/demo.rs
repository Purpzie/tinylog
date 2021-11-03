#[macro_use]
extern crate log;
use std::env;

#[test]
fn demo() {
	env::set_var("RUST_LOG", "trace");
	env::set_var("FORCE_COLOR", "1");
	tinylog::init();
	error!("test");
	warn!("test");
	info!("test");
	debug!("test");
	trace!("test");
}
