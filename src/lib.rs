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
#![cfg_attr(docs_rs, feature(doc_auto_cfg))]
#[cfg(all(not(feature = "log"), not(feature = "tracing")))]
compile_error!("at least one of 'log' or 'tracing' features must be enabled");

mod compat;
#[cfg(feature = "log")]
mod log_impl;
#[cfg(feature = "tracing")]
mod tracing_impl;
mod util;

use crate::{
	compat::{Level, Metadata},
	util::StringLike,
};
use std::io;

#[cfg(feature = "parking_lot")]
use parking_lot::Mutex;
#[cfg(not(feature = "parking_lot"))]
use std::sync::Mutex;
#[cfg(feature = "timestamps")]
use std::time::SystemTime;

/// A tiny logger.
#[non_exhaustive]
#[derive(Debug)]
pub struct Logger<T: io::Write + Send + Sync + 'static = io::Stdout> {
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
	/// If `detect-timezone` is enabled, this defaults to the local timezone.
	/// Otherwise, this defaults to UTC.
	#[cfg(feature = "timestamps")]
	pub timezone: time::UtcOffset,
}

impl Default for Logger<io::Stdout> {
	fn default() -> Self {
		Self::new(io::stdout())
	}
}

struct PrefixOptions {
	align: bool,

	#[cfg(feature = "timestamps")]
	time: Option<SystemTime>,
}

impl<T: io::Write + Send + Sync + 'static> Logger<T> {
	/// Create a new [`Logger`].
	///
	/// # Panics
	/// Panics if there was an error getting the local timezone.
	/// (Only if `detect-timezone` is enabled).
	pub fn new(output: T) -> Self {
		Self {
			output: Mutex::new(output),

			#[cfg(not(feature = "detect-color"))]
			color: false,

			#[cfg(feature = "detect-color")]
			color: supports_color::on(supports_color::Stream::Stdout)
				.map(|i| i.has_basic)
				.unwrap_or(false),

			#[cfg(all(feature = "timestamps", feature = "detect-timezone"))]
			timezone: time::UtcOffset::current_local_offset()
				.expect("failed to get local utc offset"),

			#[cfg(all(feature = "timestamps", not(feature = "detect-timezone")))]
			timezone: time::UtcOffset::UTC,
		}
	}

	fn write_prefix<S: StringLike>(
		&self,
		output: &mut S,
		meta: &Metadata,
		options: &PrefixOptions,
	) {
		let color = self.color;

		let (icon, level_str, color_code) = match meta.level {
			Level::Trace => ('→', "trace", '4'),
			Level::Debug => ('○', "debug", '6'),
			Level::Info => ('●', "info", '2'),
			Level::Warn => ('⚠', "warn", '3'),
			Level::Error => ('✘', "error", '1'),
		};

		if options.align && matches!(meta.level, Level::Info | Level::Warn) {
			output.push(' ');
		}

		// icon
		if color {
			// bright color
			output.push_str("\x1b[9");
			output.push(color_code);
			output.push('m');
		}
		output.push(icon);
		output.push(' ');

		// level
		if color {
			// bold, underline
			output.push_str("\x1b[1;4m");
		}
		output.push_str(level_str);
		if color {
			// reset, regular color
			output.push_str("\x1b[;3");
			output.push(color_code);
			output.push('m');
		}
		output.push(' ');

		let mut module_path_parts = meta.module_path.split("::");
		if let Some(first_part) = module_path_parts.next() {
			output.push_str(first_part);
			for part in module_path_parts {
				output.push('/');
				output.push_str(part);
			}
		}

		if let Some(line) = meta.line {
			if color {
				// dim
				output.push_str("\x1b[2m");
			}
			output.push(':');
			output.push_str(itoa::Buffer::new().format(line));
		}

		#[cfg(feature = "timestamps")]
		if let Some(time) = options.time {
			let time = time::OffsetDateTime::from(time).to_offset(self.timezone);
			output.push(' ');
			if color {
				// reset, dim
				output.push_str("\x1b[;2m");
			}

			// this is the only place we ever format dates. we don't really need time's formatting feature
			let mut hour = time.hour();
			let mut am_or_pm = 'A';
			if hour >= 12 {
				am_or_pm = 'P';
				if hour != 12 {
					hour -= 12;
				}
			}
			output.push_str(itoa::Buffer::new().format(hour));
			output.push(':');
			let minute = time.minute();
			if minute < 10 {
				output.push('0');
			}
			output.push_str(itoa::Buffer::new().format(minute));
			output.push(':');
			let second = time.second();
			if second < 10 {
				output.push('0');
			}
			output.push_str(itoa::Buffer::new().format(second));
			output.push('-');
			output.push(am_or_pm);
			output.push_str("M-");
			output.push_str(itoa::Buffer::new().format(time.year()));
			output.push('/');
			output.push_str(itoa::Buffer::new().format(time.month() as u8));
			output.push('/');
			output.push_str(itoa::Buffer::new().format(time.day()));
		}

		if color {
			// reset
			output.push_str("\x1b[m");
		}
	}
}
