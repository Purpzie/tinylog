use crate::Logger;
#[cfg(feature = "log")]
use log::SetLoggerError;
use std::{error::Error, fmt::Display, io};
#[cfg(feature = "tracing")]
use tracing_subscriber::{prelude::*, util::TryInitError};

/// Equivalent to [`Logger::new`]
pub fn new<T>(output: T) -> Logger<T> {
	Logger::new(output)
}

/// Equivalent to [`Logger::new_stdout`]
pub fn new_stdout() -> Logger {
	Logger::new_stdout()
}

/// Set up logging.
///
/// Equivalent to:
/// ```
/// tinylog::setup_logging_to(std::io::stdout())
/// ```
#[cfg(feature = "log")]
pub fn setup_logging() -> Result<(), SetLoggerError> {
	setup_logging_to(io::stdout())
}

/// Set up logging to the specified output.
///
/// Equivalent to:
/// ```
/// # let output = std::io::stdout();
/// log::set_boxed_logger(Box::new(Logger::new(output)))
/// ```
#[cfg(feature = "log")]
pub fn setup_logging_to<T>(output: T) -> Result<(), SetLoggerError>
where
	T: io::Write + Send + 'static,
{
	log::set_boxed_logger(Box::new(Logger::new(output)))
}

/// Set up tracing to stdout.
///
/// Equivalent to:
/// ```
/// tinylog::setup_tracing_to(std::io::stdout())
/// ```
#[cfg(feature = "tracing")]
pub fn setup_tracing() -> Result<(), TryInitError> {
	setup_tracing_to(io::stdout())
}

/// Set up tracing to the specified output.
///
/// Equivalent to:
/// ```
/// # use tracing_subscriber::prelude::*;
/// # let output = std::io::stdout();
/// tracing_subscriber::registry()
/// 	.with(Logger::new(output))
/// 	.try_init()
/// ```
#[cfg(feature = "tracing")]
pub fn setup_tracing_to<T>(output: T) -> Result<(), TryInitError>
where
	T: io::Write + Send + 'static,
{
	tracing_subscriber::registry()
		.with(Logger::new(output))
		.try_init()
}

/// An error that may occur while setting up logging and tracing.
#[cfg(any(feature = "log", feature = "tracing"))]
#[derive(Debug)]
pub enum SetupAllError {
	/// An error that originated from [`log`].
	#[cfg(feature = "log")]
	Log(SetLoggerError),
	/// An error that originated from [`tracing`].
	#[cfg(feature = "tracing")]
	Tracing(TryInitError),
}

#[cfg(any(feature = "log", feature = "tracing"))]
impl Display for SetupAllError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			#[cfg(feature = "log")]
			Self::Log(err) => err.fmt(f),
			#[cfg(feature = "tracing")]
			Self::Tracing(err) => err.fmt(f),
		}
	}
}

#[cfg(feature = "log")]
impl From<SetLoggerError> for SetupAllError {
	fn from(value: SetLoggerError) -> Self {
		Self::Log(value)
	}
}

#[cfg(feature = "tracing")]
impl From<TryInitError> for SetupAllError {
	fn from(value: TryInitError) -> Self {
		Self::Tracing(value)
	}
}

#[cfg(any(feature = "log", feature = "tracing"))]
impl Error for SetupAllError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			#[cfg(feature = "log")]
			Self::Log(err) => Some(err),
			#[cfg(feature = "tracing")]
			Self::Tracing(err) => Some(err),
		}
	}
}

/// Set up both logging and tracing (if available) to the specified output.
#[cfg(any(feature = "log", feature = "tracing"))]
pub fn setup_all_to<T>(output: T) -> Result<(), SetupAllError>
where
	T: io::Write + Send + 'static,
{
	cfg_select! {
		all(feature = "log", feature = "tracing") => {
			let logger: &'static _ = Box::leak(Box::new(Logger::new(output)));
			log::set_logger(logger)?;
			tracing_subscriber::registry().with(logger).try_init()?;
		},
		feature = "log" => setup_logging_to(output)?,
		feature = "tracing" => setup_tracing_to(output)?,
		_ => (), // this is already a compile error elsewhere
	};
	Ok(())
}

/// Set up both logging and tracing (if available) to stdout.
#[cfg(any(feature = "log", feature = "tracing"))]
pub fn setup_all() -> Result<(), SetupAllError> {
	setup_all_to(io::stdout())
}
