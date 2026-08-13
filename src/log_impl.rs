use crate::{
	Logger, PrefixOptions,
	util::{Indented, MutexHelper, StringLike, with_local_string},
};
use log::{Log, kv};
#[cfg(feature = "timestamps")]
use std::time::SystemTime;
use std::{fmt, io};

impl<T> Log for Logger<T>
where
	T: io::Write + Send,
{
	fn enabled(&self, _: &log::Metadata) -> bool {
		true
	}

	fn flush(&self) {
		self.output
			.lock_ignore_poison()
			.flush()
			.expect("failed to flush log output");
	}

	fn log(&self, record: &log::Record) {
		#[cfg(feature = "timestamps")]
		let time = SystemTime::now();

		with_local_string(move |mut buf| {
			self.write_prefix(
				&mut buf,
				&record.into(),
				&PrefixOptions {
					align: true,
					#[cfg(feature = "timestamps")]
					time: Some(time),
				},
			);

			let mut i_buf = Indented::new(&mut buf, Self::PREFIX_LABEL_LEN);
			let args = record.args();
			match args.as_str() {
				Some(str) => {
					if !str.is_empty() {
						i_buf.reserve(1 + str.len());
						i_buf.push('\n');
						i_buf.push_str(str);
					}
				},
				None => {
					i_buf.push('\n');
					fmt::Write::write_fmt(&mut i_buf, *args).expect("fmt error");
				},
			}

			record
				.key_values()
				.visit(&mut KeyValueVisitor(i_buf))
				.expect("key value visitor failed");

			buf.push('\n');
			self.output
				.lock_ignore_poison()
				.write_all(buf.as_bytes())
				.expect("io error");
		})
	}
}

struct KeyValueVisitor<T>(Indented<T>);

impl<'kvs, T> kv::VisitSource<'kvs> for KeyValueVisitor<T>
where
	T: StringLike + fmt::Write,
{
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

impl<'v, T> kv::VisitValue<'v> for KeyValueVisitor<T>
where
	T: StringLike + fmt::Write,
{
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
