#![doc = include_str!("../README.md")]
#![doc(html_root_url = "https://docs.rs/tinylog/1.2.1")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

mod config;
mod map;
pub use self::config::Config;
use self::map::DisplayMap;

use log::{Level, LevelFilter, Log, Metadata, Record};
#[cfg(feature = "humantime")]
use std::time::SystemTime;
use std::{
	fmt,
	io::{self, Write},
};
use termcolor::{BufferedStandardStream, Color, ColorChoice, ColorSpec, WriteColor};

#[cfg(feature = "parking_lot")]
use parking_lot::Mutex;
#[cfg(not(feature = "parking_lot"))]
use std::sync::Mutex;

type FilterFunc = Box<dyn Fn(&Metadata) -> bool + Send + Sync + 'static>;
type MapFunc = Box<dyn Fn(&str, &mut fmt::Formatter) -> fmt::Result + Send + Sync + 'static>;

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
	filter: Option<FilterFunc>,
	dim: Option<FilterFunc>,
	map_target: Option<MapFunc>,
	map_content: Option<MapFunc>,
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
			dim: config.dim,
			map_target: config.map_target,
			map_content: config.map_content,
			stdout: Mutex::new(stdout),
		}
	}

	fn real_log(&self, record: &Record) -> io::Result<()> {
		#[cfg(feature = "humantime")]
		let time = SystemTime::now();

		let (color, mut should_dim) = match record.level() {
			Level::Error => (Color::Red, false),
			Level::Warn => (Color::Yellow, false),
			Level::Info => (Color::Green, false),
			Level::Debug => (Color::Blue, true),
			Level::Trace => (Color::Cyan, true),
		};

		if let Some(ref func) = self.dim {
			should_dim = func(record.metadata());
		}

		let mut color = {
			let mut spec = ColorSpec::new();
			spec.set_fg(Some(color))
				.set_bold(true)
				.set_dimmed(should_dim);
			spec
		};

		#[cfg(not(feature = "parking_lot"))]
		let mut stdout = self.stdout.lock().expect("stream poisoned");
		#[cfg(feature = "parking_lot")]
		let mut stdout = self.stdout.lock();

		#[cfg(feature = "humantime")]
		stdout.set_color(ColorSpec::new().set_dimmed(true))?;
		#[cfg(feature = "humantime")]
		write!(stdout, "{} ", humantime::format_rfc3339_seconds(time))?;
		stdout.set_color(&color)?;
		write!(stdout, "{:>5} ", record.level())?;

		stdout.set_color(&*color.set_bold(false))?;
		if let Some(ref func) = self.map_target {
			write!(stdout, "{} ", DisplayMap(func, record.target()))?;
		} else {
			write!(stdout, "({}) ", record.target())?;
		}

		stdout.set_color(&*color.set_fg(None))?;
		if let Some(ref func) = self.map_content {
			let content = format!("{}", record.args());
			writeln!(stdout, "{}", DisplayMap(func, &content))?;
		} else {
			writeln!(stdout, "{}", record.args())?;
		}

		stdout.reset()?;
		stdout.flush()
	}
}
