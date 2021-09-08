#![doc = include_str!("../README.md")]
#![doc(html_root_url = "https://docs.rs/tinylog/1.1.1")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

mod config;
pub use self::config::Config;

use log::{Level, LevelFilter, Log, Metadata, Record};
#[cfg(feature = "humantime")]
use std::time::SystemTime;
use std::{
	io::{self, Write},
	sync::Mutex,
};
use termcolor::{BufferedStandardStream, Color, ColorChoice, ColorSpec, WriteColor};

/// Initialize the logger.
///
/// Any logs that occur before this are ignored.
///
/// # Example
/// ```
/// tinylog::init();
/// ```
pub fn init() {
	config().init();
}

/// Configure logging.
///
/// See [`Config`] for information.
///
/// # Example
/// ```
/// tinylog::config()
/// 	// configure things here...
/// 	.init();
/// ```
pub fn config() -> Config {
	Config::new()
}

struct Logger {
	level: LevelFilter,
	filter: Option<Box<dyn Fn(&Metadata) -> bool + Send + Sync + 'static>>,
	stdout: Mutex<BufferedStandardStream>,
}

impl Log for Logger {
	fn enabled(&self, meta: &Metadata) -> bool {
		self.level >= meta.level()
			&& if let Some(ref f) = self.filter {
				f(meta)
			} else {
				true
			}
	}

	fn log(&self, record: &Record) {
		if self.enabled(record.metadata()) {
			self.real_log(record).expect("log error");
		}
	}

	fn flush(&self) {}
}

impl Logger {
	fn new(config: Config) -> Self {
		let color = config.color.unwrap_or(ColorChoice::Never);
		let stdout = BufferedStandardStream::stdout(color);
		Self {
			level: config.level,
			filter: config.filter,
			stdout: Mutex::new(stdout),
		}
	}

	fn real_log(&self, record: &Record) -> io::Result<()> {
		#[cfg(feature = "humantime")]
		let time = SystemTime::now();

		let (color, should_dim) = match record.level() {
			Level::Error => (Color::Red, false),
			Level::Warn => (Color::Yellow, false),
			Level::Info => (Color::Green, false),
			Level::Debug => (Color::Blue, true),
			Level::Trace => (Color::Cyan, true),
		};

		let mut color = {
			let mut spec = ColorSpec::new();
			spec.set_fg(Some(color))
				.set_bold(true)
				.set_dimmed(should_dim);
			spec
		};

		let mut stdout = self.stdout.lock().expect("stream poisoned");
		#[cfg(feature = "humantime")]
		stdout.set_color(ColorSpec::new().set_dimmed(true))?;
		#[cfg(feature = "humantime")]
		write!(stdout, "{} ", humantime::format_rfc3339_seconds(time))?;
		stdout.set_color(&color)?;
		write!(stdout, "{:>5} ", record.level())?;
		stdout.set_color(&*color.set_bold(false))?;
		write!(stdout, "({}) ", record.target())?;
		stdout.set_color(&*color.set_fg(None))?;
		writeln!(stdout, "{}", record.args())?;
		stdout.reset()?;
		stdout.flush()
	}
}
