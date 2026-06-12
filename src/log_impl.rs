use crate::{
	Logger, PrefixOptions,
	util::{Indented, StringLike, with_local_string},
};
use log::{Log, kv};
use std::{fmt, io};

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

		with_local_string(move |mut buf| {
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
					indented.reserve(1 + str.len());
					indented.push('\n');
					indented.push_str(str);
				},
				None => {
					indented.push('\n');
					fmt::Write::write_fmt(&mut indented, *args).expect("fmt error");
				},
				_ => (),
			}

			record
				.key_values()
				.visit(&mut KeyValueVisitor(indented))
				.expect("key value visitor failed");

			buf.push('\n');
			#[allow(unused_mut)]
			let mut output = self.output.lock();
			#[cfg(not(feature = "parking_lot"))]
			let mut output = output.unwrap_or_else(|e| e.into_inner());
			output.write_all(buf.as_bytes()).expect("io error");
		})
	}
}

struct KeyValueVisitor<T: StringLike + fmt::Write>(Indented<T>);

impl<'kvs, T: StringLike + fmt::Write> kv::VisitSource<'kvs> for KeyValueVisitor<T> {
	fn visit_pair(&mut self, key: kv::Key<'kvs>, value: kv::Value<'kvs>) -> Result<(), kv::Error> {
		let key_name = key.as_str();
		self.0.reserve(key_name.len() + 3);
		self.0.push('\n');
		self.0.push_str(key_name);
		self.0.push_str(": ");
		self.0.indent += 2;
		let result = value.visit(&mut *self);
		self.0.indent -= 2;
		result
	}
}

impl<'v, T: StringLike + fmt::Write> kv::VisitValue<'v> for KeyValueVisitor<T> {
	fn visit_any(&mut self, value: kv::Value) -> Result<(), kv::Error> {
		fmt::Write::write_fmt(&mut self.0, format_args!("{value:?}"))
			.map_err(|_| kv::Error::msg("fmt error"))
	}

	fn visit_null(&mut self) -> Result<(), kv::Error> {
		self.0.push_str("null");
		Ok(())
	}

	fn visit_u64(&mut self, value: u64) -> Result<(), kv::Error> {
		self.0.push_str(itoa::Buffer::new().format(value));
		Ok(())
	}

	fn visit_i64(&mut self, value: i64) -> Result<(), kv::Error> {
		self.0.push_str(itoa::Buffer::new().format(value));
		Ok(())
	}

	fn visit_u128(&mut self, value: u128) -> Result<(), kv::Error> {
		self.0.push_str(itoa::Buffer::new().format(value));
		Ok(())
	}

	fn visit_i128(&mut self, value: i128) -> Result<(), kv::Error> {
		self.0.push_str(itoa::Buffer::new().format(value));
		Ok(())
	}

	fn visit_f64(&mut self, value: f64) -> Result<(), kv::Error> {
		self.0.push_str(zmij::Buffer::new().format(value));
		Ok(())
	}

	fn visit_bool(&mut self, value: bool) -> Result<(), kv::Error> {
		self.0.push_str(if value { "true" } else { "false" });
		Ok(())
	}

	fn visit_str(&mut self, value: &str) -> Result<(), kv::Error> {
		self.0.push_str(value);
		Ok(())
	}

	fn visit_borrowed_str(&mut self, value: &'v str) -> Result<(), kv::Error> {
		self.0.push_str(value);
		Ok(())
	}

	fn visit_char(&mut self, value: char) -> Result<(), kv::Error> {
		self.0.push(value);
		Ok(())
	}
}
