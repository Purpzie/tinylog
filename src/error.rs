//! Errors for this crate.

use std::{
	fmt::{self, Debug, Display, Error as FmtError},
	io::Error as IoError,
};

/// An error that may occur while logging.
#[derive(Debug)]
#[non_exhaustive]
pub enum LogError {
	/// A [`fmt::Error`] occurred.
	Fmt(FmtError),
	/// An [`io::Error`](std::io::Error) occurred.
	Io(IoError),
}

impl From<FmtError> for LogError {
	fn from(err: FmtError) -> Self {
		Self::Fmt(err)
	}
}

impl From<IoError> for LogError {
	fn from(err: IoError) -> Self {
		Self::Io(err)
	}
}

impl Display for LogError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str("log write error: ")?;
		match self {
			Self::Fmt(ref err) => Display::fmt(err, f),
			Self::Io(ref err) => Display::fmt(err, f),
		}
	}
}

impl std::error::Error for LogError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		Some(match self {
			Self::Fmt(ref err) => err,
			Self::Io(ref err) => err,
		})
	}
}
