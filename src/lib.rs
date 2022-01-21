#![doc = include_str!("../README.md")]
#![forbid(unsafe_code, rustdoc::broken_intra_doc_links)]
#![warn(
	missing_docs,
	clippy::missing_docs_in_private_items,
	missing_debug_implementations
)]
#![allow(clippy::tabs_in_doc_comments)]

mod config;
mod error;
mod write;
use crate::write::Writer;
pub use crate::{config::Config, error::LogError, write::Formatter};

use log::{Level, LevelFilter, Log, Metadata, Record};
#[cfg(feature = "humantime")]
use std::time::SystemTime;
use std::{
	cell::RefCell,
	env,
	fmt::{self, Write as _},
	str::FromStr,
};
use termcolor::{Color, ColorChoice, ColorSpec};

#[cfg(test)]
mod tests;

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

/// Alias for a function that uses a [`Formatter`].
type FormatFn = Box<dyn Fn(&Record, &mut Formatter) -> fmt::Result + Send + Sync>;

#[allow(clippy::missing_docs_in_private_items)]
struct Logger {
	level: LevelFilter,
	output: Writer,
	filter: Option<Box<dyn Fn(&Metadata) -> bool + Send + Sync>>,
	dim: Option<Box<dyn Fn(&Record) -> bool + Send + Sync>>,
	display_level: Option<FormatFn>,
	display_target: Option<FormatFn>,
	display_content: Option<FormatFn>,
	on_error: Option<Box<dyn Fn(LogError) + Send + Sync>>,
}

impl Logger {
	/// Run on initialization.
	fn init(mut config: Config) {
		if config.level_is_default {
			// can be overridden with this env var
			if let Ok(level_str) = env::var("RUST_LOG") {
				config.level = LevelFilter::from_str(&level_str)
					.expect("RUST_LOG must be 'error', 'warn', 'info', 'debug', 'trace', or 'off'")
			}
		}

		if config.color_choice == ColorChoice::Auto {
			if env::var("FORCE_COLOR").is_ok() {
				config.color_choice = ColorChoice::Always;
			} else if env::var("NO_COLOR").is_ok()
				|| cfg!(not(test)) && atty::isnt(atty::Stream::Stdout)
			//   ^^^^^^^^^^^^^^^ this is why most tests are in ./tests.rs
			{
				config.color_choice = ColorChoice::Never;
			}
		}

		let logger = Self {
			level: config.level,
			output: Writer::new(config.color_choice),
			filter: config.filter,
			dim: config.dim,
			display_level: config.display_level,
			display_target: config.display_target,
			display_content: config.display_content,
			on_error: config.on_error,
		};

		log::set_max_level(config.level);
		log::set_boxed_logger(Box::new(logger)).expect("logger already set");
	}

	/// The true logging function.
	fn real_log(&self, buf: &mut Formatter, record: &Record) -> Result<(), LogError> {
		let mut color = ColorSpec::new();

		// timestamp
		#[cfg(feature = "humantime")]
		{
			let time = SystemTime::now();
			buf.set_color(color.set_dimmed(true))?;
			write!(buf, "{} ", humantime::format_rfc3339_seconds(time))?;
		}

		color
			.set_bold(true)
			.set_fg(Some(match record.level() {
				Level::Error => Color::Red,
				Level::Warn => Color::Yellow,
				Level::Info => Color::Green,
				Level::Debug => Color::Blue,
				Level::Trace => Color::Magenta,
			}))
			.set_dimmed(match self.dim {
				None => record.level() >= Level::Debug,
				Some(ref f) => f(record),
			});

		// level
		buf.set_color(&color)?;
		match self.display_level {
			None => write!(buf, "{:>5}", record.level()),
			Some(ref f) => f(record, buf),
		}?;

		// target
		buf.set_color(color.set_bold(false))?;
		match self.display_target {
			None => write!(buf, " ({}) ", record.target()),
			Some(ref f) => f(record, buf),
		}?;

		// content
		buf.set_color(color.set_fg(None))?;
		match self.display_content {
			None => buf.write_fmt(*record.args()),
			Some(ref f) => f(record, buf),
		}?;

		buf.reset_color()?;
		buf.write_str("\n")?;
		self.output.print(buf)?;
		Ok(())
	}

	/// Forces a log to occur even if the thread-local formatter isn't available.
	fn force_log(&self, record: &Record) -> Result<(), LogError> {
		self.real_log(&mut self.output.new_formatter(), record)
	}
}

impl Log for Logger {
	fn enabled(&self, meta: &Metadata) -> bool {
		let level_enabled = self.level >= meta.level();
		if level_enabled {
			if let Some(ref f) = self.filter {
				return f(meta);
			}
		}
		level_enabled
	}

	fn log(&self, record: &Record) {
		if !self.enabled(record.metadata()) {
			return;
		}

		// instead of allocating a formatter (which is just a buffer) for every log, use one per thread
		// see https://github.com/env-logger-rs/env_logger/blob/cb5375c/src/lib.rs#L922
		thread_local! {
			static BUFFER: RefCell<Option<Formatter>> = RefCell::new(None);
		}

		let result = BUFFER
			.try_with(|cell| {
				match cell.try_borrow_mut() {
					Ok(mut option) => {
						let buf = option.get_or_insert_with(|| self.output.new_formatter());
						buf.clear();
						self.real_log(buf, record)
					}

					// already borrowed
					Err(_) => self.force_log(record),
				}
			})
			// access error
			.unwrap_or_else(|_| self.force_log(record));

		if let Err(err) = result {
			if let Some(ref f) = &self.on_error {
				f(err)
			}
		}
	}

	fn flush(&self) {
		// nothing
	}
}
