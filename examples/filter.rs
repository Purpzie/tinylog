#[macro_use]
extern crate log;

fn main() {
	tinylog::config()
		.filter(|meta| meta.target() != "bad")
		.init();

	info!(target: "foo", "foo");
	error!(target: "bad", "you shouldn't see this");
	info!(target: "bar", "bar");
	error!(target: "bad", "you shouldn't see this");
	info!(target: "baz", "baz");
}
