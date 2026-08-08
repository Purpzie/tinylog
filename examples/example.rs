#[macro_use]
extern crate log;

fn main() {
	log::set_max_level(log::LevelFilter::Trace);
	log::set_boxed_logger(Box::new(tinylog::Logger::default())).unwrap();
	error!("hello world");
	warn!("hello world");
	info!("hello world");
	debug!("hello world");
	trace!("hello world");
}
