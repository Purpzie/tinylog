#[cfg(debug_assertions)]
const DEFAULT_LEVEL: LevelFilter = LevelFilter::Debug;
#[cfg(not(debug_assertions))]
const DEFAULT_LEVEL: LevelFilter = LevelFilter::Info;

use super::Logger;
use log::{LevelFilter, Metadata};
use std::{
	env::{self, VarError},
	fmt,
	str::FromStr,
};
use termcolor::ColorChoice;

/// Used to configure logging.
///
/// This is returned from [`tinylog::config()`](crate::config()).
pub struct Config {
	pub(super) level: LevelFilter,
	level_is_default: bool,
	pub(super) color: Option<ColorChoice>,
	pub(super) filter: Option<Box<dyn Fn(&Metadata) -> bool + Send + Sync + 'static>>,
}

// can't derive because of trait object
impl fmt::Debug for Config {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Config")
			.field("level", &self.level)
			.field("level_is_default", &self.level_is_default)
			.field("color", &self.color)
			.field(
				"filter",
				&if self.filter.is_some() {
					Some(())
				} else {
					None
				},
			)
			.finish()
	}
}

impl Config {
	pub(super) fn new() -> Self {
		Self {
			level: DEFAULT_LEVEL,
			level_is_default: true,
			color: None,
			filter: None,
		}
	}

	/// Set the default log level.
	///
	/// If `RUST_LOG` is set, it will override this.
	/// You can use [`Config::level`] to ignore `RUST_LOG`.
	///
	/// # Example
	/// ```
	/// # use log::LevelFilter;
	/// tinylog::config()
	/// 	.default_level(LevelFilter::Warn)
	/// 	.init();
	/// ```
	pub fn default_level(mut self, level: LevelFilter) -> Self {
		self.level = level;
		self.level_is_default = true;
		self
	}

	/// Set the log level, ignoring `RUST_LOG`.
	///
	/// # Example
	/// ```
	/// # use log::LevelFilter;
	/// tinylog::config()
	/// 	.level(LevelFilter::Warn)
	/// 	.init();
	/// ```
	pub fn level(mut self, level: LevelFilter) -> Self {
		self.level = level;
		self.level_is_default = false;
		self
	}

	/// Filter logs.
	///
	/// # Example
	/// ```
	/// tinylog::config()
	/// 	.filter(|metadata| {
	/// 		// only show logs from this binary
	/// 		metadata.target().starts_with(module_path!())
	/// 	})
	/// 	.init();
	/// ```
	pub fn filter<F>(mut self, filter: F) -> Self
	where
		F: Fn(&Metadata) -> bool + Send + Sync + 'static,
	{
		self.filter = Some(Box::new(filter));
		self
	}

	/// Set the color level.
	///
	/// If this is set, color support won't be determined with [atty]. The environment variables
	/// `NO_COLOR` and `FORCE_COLOR` will still apply, however.
	///
	/// # Example
	/// ```
	/// # use termcolor::ColorChoice;
	/// tinylog::config()
	/// 	// never output colors
	/// 	.color(ColorChoice::Never)
	/// 	.init();
	/// ```
	pub fn color(mut self, color: ColorChoice) -> Self {
		self.color = Some(color);
		self
	}

	/// Initialize the logger.
	///
	/// Any logs that occur before this are ignored.
	///
	/// # Example
	/// ```
	/// // equivalent to tinylog::init();
	/// tinylog::config().init();
	/// ```
	pub fn init(mut self) {
		self.options_init();
		log::set_max_level(self.level);
		log::set_boxed_logger(Box::new(Logger::new(self))).unwrap();
	}

	fn options_init(&mut self) {
		if env::var("FORCE_COLOR").is_ok() {
			self.color = Some(ColorChoice::Always);
		} else if env::var("NO_COLOR").is_ok() {
			self.color = Some(ColorChoice::Never);
		} else if self.color.is_none() && atty::is(atty::Stream::Stdout) {
			self.color = Some(ColorChoice::Auto);
		}

		if self.level_is_default {
			match env::var("RUST_LOG") {
				Err(err) => match err {
					VarError::NotPresent => (),
					VarError::NotUnicode(_) => panic!("RUST_LOG must be valid unicode"),
				},

				Ok(value) => {
					self.level = LevelFilter::from_str(&value)
						.expect("RUST_LOG must be one of 'error, warn, info, debug, trace, off'")
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rust_log() {
		env::set_var("RUST_LOG", "trace");
		let mut config = Config::new();
		config.options_init();
		assert_eq!(config.level, LevelFilter::Trace);
		env::set_var("RUST_LOG", "warn");
		config.options_init();
		assert_eq!(config.level, LevelFilter::Warn);
	}

	#[test]
	fn ignore_rust_log() {
		env::set_var("RUST_LOG", "warn");
		let mut config = Config::new().level(LevelFilter::Off);
		config.options_init();
		assert_eq!(config.level, LevelFilter::Off);
	}

	#[test]
	fn default_level() {
		env::remove_var("RUST_LOG");
		let mut config = Config::new().default_level(LevelFilter::Trace);
		config.options_init();
		assert_eq!(config.level, LevelFilter::Trace);
		env::set_var("RUST_LOG", "error");
		config.options_init();
		assert_eq!(config.level, LevelFilter::Error);
	}

	#[test]
	fn set_color_level() {
		env::remove_var("FORCE_COLOR");
		env::remove_var("NO_COLOR");
		let mut config = Config::new().color(ColorChoice::AlwaysAnsi);
		config.options_init();
		assert_eq!(config.color, Some(ColorChoice::AlwaysAnsi));
	}

	#[test]
	fn force_color() {
		env::set_var("FORCE_COLOR", "1");
		let mut config = Config::new().color(ColorChoice::Never);
		config.options_init();
		assert_eq!(config.color, Some(ColorChoice::Always));
	}

	#[test]
	fn no_color() {
		env::remove_var("FORCE_COLOR");
		env::set_var("NO_COLOR", "1");
		let mut config = Config::new().color(ColorChoice::Always);
		config.options_init();
		assert_eq!(config.color, Some(ColorChoice::Never));
	}

	#[test]
	fn filter() {
		let config = Config::new().filter(|_| true);
		if let Some(ref f) = config.filter {
			let meta = Metadata::builder().build();
			assert!(f(&meta));
		} else {
			panic!("config.filter is None");
		}
	}
}
