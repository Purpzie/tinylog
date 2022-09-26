use crate::{
	util::{with_local_buf, Indented, StringLike},
	Logger, PrefixOptions,
};
use log::Log;
use std::{fmt::Write, io};

#[cfg(feature = "timestamps")]
use std::time::SystemTime;

impl<T: io::Write + Send + Sync + 'static> Log for Logger<T> {
	fn enabled(&self, _: &log::Metadata) -> bool {
		true
	}

	fn flush(&self) {
		#[allow(unused_mut)]
		let mut output = self.output.lock();
		#[cfg(not(feature = "parking_lot"))]
		let mut output = output.unwrap_or_else(|e| e.into_inner());
		output.flush().expect("failed to flush log output");
	}

	fn log(&self, record: &log::Record) {
		#[cfg(feature = "timestamps")]
		let time = SystemTime::now();

		with_local_buf(move |mut buf| {
			buf.clear();

			self.write_prefix(
				&mut buf,
				&record.into(),
				&PrefixOptions {
					align: true,
					#[cfg(feature = "timestamps")]
					time: Some(time),
				},
			);

			let mut indented = Indented::new(&mut buf, 8);
			let args = record.args();
			match args.as_str() {
				Some(str) if !str.is_empty() => {
					indented.push('\n');
					indented.push_str(str);
				},
				None => {
					indented.push('\n');
					indented.write_fmt(*args).expect("fmt error");
				},
				_ => (),
			}

			buf.push('\n');
			#[allow(unused_mut)]
			let mut output = self.output.lock();
			#[cfg(not(feature = "parking_lot"))]
			let mut output = output.unwrap_or_else(|e| e.into_inner());
			output.write_all(buf.as_bytes()).expect("io error");
		})
	}
}
