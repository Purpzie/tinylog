#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Level {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
}

pub(super) struct Metadata<'a> {
	pub level: Level,
	pub module_path: &'a str,
	pub line: Option<u32>,
}

#[cfg(feature = "log")]
impl From<log::Level> for Level {
	fn from(level: log::Level) -> Self {
		match level {
			log::Level::Trace => Level::Trace,
			log::Level::Debug => Level::Debug,
			log::Level::Info => Level::Info,
			log::Level::Warn => Level::Warn,
			log::Level::Error => Level::Error,
		}
	}
}

#[cfg(feature = "log")]
impl<'a> From<&log::Record<'a>> for Metadata<'a> {
	fn from(record: &log::Record<'a>) -> Self {
		Self {
			level: record.level().into(),
			module_path: record.module_path().unwrap_or_else(|| record.target()),
			line: record.line(),
		}
	}
}

#[cfg(feature = "tracing")]
impl From<tracing::Level> for Level {
	fn from(level: tracing::Level) -> Self {
		match level {
			tracing::Level::TRACE => Level::Trace,
			tracing::Level::DEBUG => Level::Debug,
			tracing::Level::INFO => Level::Info,
			tracing::Level::WARN => Level::Warn,
			tracing::Level::ERROR => Level::Error,
		}
	}
}

#[cfg(feature = "tracing")]
impl<'a> From<&tracing::Metadata<'a>> for Metadata<'a> {
	fn from(metadata: &tracing::Metadata<'a>) -> Self {
		Self {
			level: (*metadata.level()).into(),
			module_path: metadata.module_path().unwrap_or_else(|| metadata.target()),
			line: metadata.line(),
		}
	}
}
