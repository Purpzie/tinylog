//! Stuff for writing to stdout.

use std::{
	fmt::{self, Debug},
	io::{self, Write as _},
};
use termcolor::{Buffer, BufferWriter, ColorChoice, ColorSpec, WriteColor};

/// Use this to print text and colors to stdout.
pub(super) struct Writer(BufferWriter);

impl Writer {
	/// Create a new writer.
	pub fn new(color_choice: ColorChoice) -> Self {
		Self(BufferWriter::stdout(color_choice))
	}

	/// Create a formatter for this writer.
	pub fn new_formatter(&self) -> Formatter {
		Formatter(self.0.buffer())
	}

	/// Print the results of a formatter to stdout.
	pub fn print(&self, formatter: &Formatter) -> io::Result<()> {
		self.0.print(&formatter.0)
	}
}

/// Used with display functions.
///
/// This implements [`fmt::Write`].
pub struct Formatter(Buffer);

impl Debug for Formatter {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Formatter").finish_non_exhaustive()
	}
}

impl fmt::Write for Formatter {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		match self.0.write_all(s.as_bytes()) {
			Ok(_) => Ok(()),
			Err(_) => Err(fmt::Error),
		}
	}
}

#[allow(clippy::missing_docs_in_private_items)]
impl Formatter {
	pub(super) fn set_color(&mut self, color: &ColorSpec) -> io::Result<()> {
		self.0.set_color(color)
	}

	pub(super) fn reset_color(&mut self) -> io::Result<()> {
		self.0.reset()
	}

	pub(super) fn clear(&mut self) {
		self.0.clear();
	}
}
