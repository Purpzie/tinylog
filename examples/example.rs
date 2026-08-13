fn main() {
	log::set_max_level(log::LevelFilter::Trace);
	tinylog::setup_all().unwrap();
	log::error!("hello world");
	log::warn!("hello world");
	log::info!("hello world");
	log::debug!("hello world");
	log::trace!("hello world");

	tracing_test();
}

#[tracing::instrument]
fn tracing_test() {
	tracing::info!("tracing");
}
