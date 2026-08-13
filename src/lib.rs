//! ## Configuration
//! Output, color, and timezone can be configured on the [`Logger`].
//!
//! #### Features
//! - `detect-color` - Automatically detect terminal color support.
//! - `detect-timezone` - If `timestamps` are enabled, the local timezone will automatically be detected and used.
//! - `log` - Enable [`log`] support.
//! - `parking_lot` - Use [`parking_lot`] internally.
//! - `timestamps` - Enable timestamps.
//! - `tracing` - Enable [`tracing`] support.
//!
//! #### Log level
//! Set the level using `log` or `tracing` directly.
//!
//! #### Filtering
//! To add filtering with [`log`], create a new struct that implements `Log::enabled`, and forward
//! the other methods to `tinylog::Logger`.
//!
//! For [`tracing`], [`tracing_subscriber`] already lets you add filters to existing layers.

#![forbid(unsafe_code)]
#![allow(clippy::tabs_in_doc_comments)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(not(feature = "log"), not(feature = "tracing")))]
compile_error!("at least one of 'log' or 'tracing' features must be enabled");

mod compat;
mod helpers;
#[cfg(feature = "log")]
mod log_impl;
#[cfg(feature = "tracing")]
mod tracing_impl;
mod util;

pub use crate::helpers::*;
use crate::{
	compat::{Level, Metadata},
	util::StringLike,
};
use std::io;
#[cfg(feature = "timestamps")]
use std::time::SystemTime;

#[cfg(feature = "parking_lot")]
use parking_lot::Mutex;
#[cfg(not(feature = "parking_lot"))]
use std::sync::Mutex;

/// A tiny logger.
#[non_exhaustive]
#[derive(Debug)]
pub struct Logger<T = io::Stdout> {
	output: Mutex<T>,

	/// Whether color should be enabled.
	///
	/// Defaults to [`false`](bool) if `detect-color` is ***not*** enabled.
	///
	/// Note: `detect-color` only checks [`io::Stdout`] for color support.
	/// If you set the output to something else, you should disable `detect-color`.
	pub color: bool,

	/// The timezone to display timestamps in.
	///
	/// If `detect-timezone` is enabled, this defaults to the local timezone if available.
	/// Otherwise, this defaults to UTC.
	#[cfg(feature = "timestamps")]
	pub timezone: time::UtcOffset,
}

impl<T> Logger<T> {
	/// Create a new [`Logger`] that writes to `output`.
	pub fn new(output: T) -> Self {
		Self {
			output: Mutex::new(output),

			color: cfg_select! {
				feature = "detect-color" => {
					supports_color::on(supports_color::Stream::Stdout)
						.map(|i| i.has_basic)
						.unwrap_or(false)
				},
				_ => false,
			},

			#[cfg(feature = "timestamps")]
			timezone: cfg_select! {
				feature = "detect-timezone" => {
					time::UtcOffset::current_local_offset()
						.unwrap_or(time::UtcOffset::UTC)
				},
				_ => time::UtcOffset::UTC,
			},
		}
	}
}

impl Logger {
	/// Equivalent to:
	/// ```
	/// Logger::new(std::io::stdout())
	/// ```
	#[allow(clippy::new_without_default)]
	pub fn new_stdout() -> Self {
		Self::new(io::stdout())
	}
}

struct PrefixOptions {
	align: bool,

	#[cfg(feature = "timestamps")]
	time: Option<SystemTime>,
}

impl<T> Logger<T>
where
	T: io::Write,
{
	const PREFIX_LABEL_LEN: usize = 8; // icon, space, label (5), space

	/// Write the logging level, module info, and timestamp (if configured) for a message.
	fn write_prefix<S: StringLike>(
		&self,
		output: &mut S,
		meta: &Metadata,
		options: &PrefixOptions,
	) {
		let mut reserve_len = Self::PREFIX_LABEL_LEN;

		// approximate. :: gets replaced with / and we append a : and the line number
		// assume bad case with no :: and line number with 4 digits
		reserve_len += meta.module_path.len() + 5;

		if cfg!(feature = "timestamps") {
			// space + 00:00:00AM 0000/00/00
			reserve_len += 22;
		}

		if self.color {
			// unfortunately counted by hand. keep up to date!
			reserve_len += 24;
			if cfg!(feature = "timestamps") {
				reserve_len += 5;
			}
		}

		output.reserve(reserve_len);

		if options.align && matches!(meta.level, Level::Info | Level::Warn) {
			output.push(' ');
		}

		// https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters

		let (icon, level_str, color_code) = match meta.level {
			Level::Trace => ('→', "trace", '5'),
			Level::Debug => ('○', "debug", '6'),
			Level::Info => ('●', "info", '2'),
			Level::Warn => ('⚠', "warn", '3'),
			Level::Error => ('✘', "error", '1'),
		};

		// icon
		if self.color {
			// bright color
			output.push_str("\x1b[9");
			output.push(color_code);
			output.push('m');
		}
		output.push(icon);

		// level
		output.push(' ');
		if self.color {
			// bold, underline
			output.push_str("\x1b[1;4m");
		}
		output.push_str(level_str);

		let mut int_buf = itoa::Buffer::new();

		#[cfg(feature = "timestamps")]
		if let Some(time) = options.time {
			if self.color {
				// reset, dim
				output.push_str("\x1b[;2m");
			}
			output.push(' ');

			let time = time::OffsetDateTime::from(time).to_offset(self.timezone);

			// this is the only place we ever format dates.
			// time's formatting always allocates, so let's just do it manually instead.
			let mut hour = time.hour();
			let mut is_pm = false;
			match hour {
				0 => hour = 12,
				13.. => {
					hour -= 12;
					is_pm = true;
				},
				_ => (),
			};
			output.push_str(int_buf.format(hour));
			let minute = time.minute();
			output.push(':');
			if minute < 10 {
				output.push('0');
			}
			output.push_str(int_buf.format(minute));
			let second = time.second();
			output.push(':');
			if second < 10 {
				output.push('0');
			}
			output.push_str(int_buf.format(second));
			output.push(if is_pm { 'P' } else { 'A' });
			output.push_str("M ");

			output.push_str(int_buf.format(time.year()));
			output.push('/');
			output.push_str(int_buf.format(time.month() as u8));
			output.push('/');
			output.push_str(int_buf.format(time.day()));
		}

		if self.color {
			// reset, regular color
			output.push_str("\x1b[;3");
			output.push(color_code);
			output.push('m');
		}
		output.push(' ');

		// personal opinion: / is better than ::
		let mut module_path_parts = meta.module_path.split("::");
		if let Some(first_part) = module_path_parts.next() {
			output.push_str(first_part);
			for part in module_path_parts {
				output.push('/');
				output.push_str(part);
			}
		}

		if let Some(line) = meta.line {
			if self.color {
				// dim
				output.push_str("\x1b[2m");
			}
			output.push(':');
			output.push_str(int_buf.format(line));
		}

		if self.color {
			// reset
			output.push_str("\x1b[m");
		}
	}
}
