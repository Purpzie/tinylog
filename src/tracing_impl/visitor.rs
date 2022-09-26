use crate::util::StringLike;
use std::fmt;
use tracing::field::{Field, Visit};

pub(super) struct FieldVisitor<T: StringLike + fmt::Write>(T);

impl<T: StringLike + fmt::Write> FieldVisitor<T> {
	pub fn new(output: T) -> Self {
		Self(output)
	}

	fn write_field<'a>(&mut self, field: &'a Field) -> &'a str {
		self.0.push('\n');
		let name = field.name();
		if name != "message" {
			self.0.push_str(name);
			self.0.push_str(": ");
		}
		name
	}
}

impl<T: StringLike + fmt::Write> Visit for FieldVisitor<T> {
	fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
		self.write_field(field);
		write!(self.0, "{value:?}").expect("fmt error");
	}

	fn record_str(&mut self, field: &Field, value: &str) {
		let name = self.write_field(field);
		if name == "message" {
			self.0.push_str(value);
		} else {
			write!(self.0, "{value:?}").expect("fmt error");
		}
	}

	fn record_bool(&mut self, field: &Field, value: bool) {
		self.write_field(field);
		self.0.push_str(if value { "true" } else { "false" });
	}

	fn record_u64(&mut self, field: &Field, value: u64) {
		self.write_field(field);
		self.0.push_str(itoa::Buffer::new().format(value));
	}

	fn record_u128(&mut self, field: &Field, value: u128) {
		self.write_field(field);
		self.0.push_str(itoa::Buffer::new().format(value));
	}

	fn record_i64(&mut self, field: &Field, value: i64) {
		self.write_field(field);
		self.0.push_str(itoa::Buffer::new().format(value));
	}

	fn record_i128(&mut self, field: &Field, value: i128) {
		self.write_field(field);
		self.0.push_str(itoa::Buffer::new().format(value));
	}

	fn record_f64(&mut self, field: &Field, value: f64) {
		self.write_field(field);
		self.0.push_str(ryu::Buffer::new().format(value));
	}
}
