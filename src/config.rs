use crate::{error::LogError, FormatFn, Logger};
use log::{LevelFilter, Metadata, Record};
use std::fmt::{self, Debug};
use termcolor::ColorChoice;

/// Configure logging.
///
/// This is returned from [`tinylog::config()`](crate::config()).
pub struct Config {
	pub(super) level: LevelFilter,
	pub(super) level_is_default: bool,
	pub(super) color_choice: ColorChoice,
	pub(super) filter: Option<Box<dyn Fn(&Metadata) -> bool + Send + Sync>>,
	pub(super) dim: Option<Box<dyn Fn(&Record) -> bool + Send + Sync>>,
	pub(super) display_level: Option<FormatFn>,
	pub(super) display_target: Option<FormatFn>,
	pub(super) display_content: Option<FormatFn>,
	pub(super) on_error: Option<Box<dyn Fn(LogError) + Send + Sync>>,
}

// can't derive because of trait objects
impl Debug for Config {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// at least show whether something is present
		struct DisplayOpt<'a, T>(&'a Option<T>);

		impl<T> Debug for DisplayOpt<'_, T> {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				f.write_str(match self.0 {
					Some(_) => "Some(_)",
					None => "None",
				})
			}
		}

		f.debug_struct("Config")
			.field("level", &self.level)
			.field("color_choice", &self.color_choice)
			.field("filter", &DisplayOpt(&self.filter))
			.field("dim", &DisplayOpt(&self.dim))
			.field("display_level", &DisplayOpt(&self.display_level))
			.field("display_target", &DisplayOpt(&self.display_target))
			.field("display_content", &DisplayOpt(&self.display_content))
			.field("handle_errors", &DisplayOpt(&self.on_error))
			.finish()
	}
}

impl Config {
	pub(super) fn new() -> Self {
		let level = if cfg!(debug_assertions) {
			LevelFilter::Debug
		} else {
			LevelFilter::Info
		};

		Self {
			level,
			level_is_default: true,
			color_choice: ColorChoice::Auto,
			filter: None,
			dim: None,
			display_level: None,
			display_target: None,
			display_content: None,
			on_error: None,
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

	/// Disable colors.
	///
	/// # Example
	/// ```
	/// tinylog::config()
	/// 	.no_color()
	/// 	.init();
	/// ```
	pub fn no_color(mut self) -> Self {
		self.color_choice = ColorChoice::Never;
		self
	}

	/// Always output colors.
	///
	/// # Example
	/// ```
	/// tinylog::config()
	/// 	.force_color()
	/// 	.init();
	/// ```
	pub fn force_color(mut self) -> Self {
		self.color_choice = ColorChoice::Always;
		self
	}

	/// Dim certain logs.
	///
	/// By default, `debug` and `trace` logs are dimmed.
	/// Note that dimming doesn't work in a windows console.
	///
	/// # Example
	/// ```
	/// tinylog::config()
	/// 	.dim(|metadata| {
	/// 		// dim all logs from external crates
	/// 		!metadata.target().starts_with(module_path!())
	/// 	})
	/// 	.init();
	/// ```
	pub fn dim<F>(mut self, dim: F) -> Self
	where
		F: Fn(&Record) -> bool + Send + Sync + 'static,
	{
		self.dim = Some(Box::new(dim));
		self
	}

	/// Modify the level format.
	///
	/// # Example
	/// ```
	/// # use log::Level;
	/// # use std::fmt::Write;
	/// tinylog::config()
	/// 	.display_level(|record, f| {
	/// 		// use lowercase instead of uppercase
	/// 		f.write_str(match record.level() {
	/// 			Level::Error => "error",
	/// 			Level::Warn => "warn",
	/// 			Level::Info => "info",
	/// 			Level::Debug => "debug",
	/// 			Level::Trace => "trace",
	/// 		})
	/// 	})
	/// 	.init();
	/// ```
	pub fn display_level<F>(mut self, display_level: F) -> Self
	where
		F: Fn(&Record, &mut crate::Formatter) -> fmt::Result + Send + Sync + 'static,
	{
		self.display_level = Some(Box::new(display_level));
		self
	}

	/// Modify the target format.
	///
	/// By default, this is ` (target) `.
	///
	/// # Example
	/// ```
	/// use std::fmt::Write;
	///
	/// tinylog::config()
	/// 	// use square brackets instead of round
	/// 	.display_target(|record, f| {
	/// 		write!(f, " [{}] ", record.target())
	/// 	})
	/// 	.init();
	/// ```
	pub fn display_target<F>(mut self, display_target: F) -> Self
	where
		F: Fn(&Record, &mut crate::Formatter) -> fmt::Result + Send + Sync + 'static,
	{
		self.display_target = Some(Box::new(display_target));
		self
	}

	/// Modify the log content.
	///
	/// # Example
	/// ```
	/// use std::fmt::Write;
	///
	/// tinylog::config()
	/// 	// make the entire message uppercase
	/// 	.display_content(move |record, f| {
	/// 		let mut str = format!("{}", record.args());
	/// 		str.make_ascii_uppercase();
	/// 		f.write_str(&str)
	/// 	})
	/// 	.init();
	/// ```
	pub fn display_content<F>(mut self, display_content: F) -> Self
	where
		F: Fn(&Record, &mut crate::Formatter) -> fmt::Result + Send + Sync + 'static,
	{
		self.display_content = Some(Box::new(display_content));
		self
	}

	/// Inspect any format/writing errors that occur.
	///
	/// Normally, these errors are ignored.
	///
	/// # Example
	/// ```should_panic
	/// # use std::fmt::{self, Display};
	/// tinylog::config()
	/// 	.on_error(|err| {
	/// 		// maybe print it or log it to a file instead, etc
	/// 		panic!("{}", err);
	/// 	})
	/// 	.init();
	///
	/// struct Invalid;
	///
	/// impl Display for Invalid {
	/// 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	/// 		Err(fmt::Error)
	/// 	}
	/// }
	///
	/// log::info!("{}", Invalid);
	/// ```
	pub fn on_error<F>(mut self, handle_errors: F) -> Self
	where
		F: Fn(LogError) + Send + Sync + 'static,
	{
		self.on_error = Some(Box::new(handle_errors));
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
	pub fn init(self) {
		Logger::init(self)
	}
}
