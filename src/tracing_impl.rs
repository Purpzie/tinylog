use crate::{
	Logger, PrefixOptions,
	util::{Indented, StringLike, with_local_string},
};
use std::{fmt, io};
use tracing::{
	Event, Id, Subscriber,
	field::{Field, Visit},
	span::{Attributes, Record},
};
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};

#[cfg(feature = "timestamps")]
use std::time::SystemTime;

struct SpanData {
	content: String,
	prefix_end_index: usize,
}

impl<S, T: io::Write + Send + Sync + 'static> Layer<S> for Logger<T>
where
	S: Subscriber + for<'any> LookupSpan<'any>,
{
	fn on_new_span(&self, attrs: &Attributes, id: &Id, ctx: Context<S>) {
		let span = ctx.span(id).expect("span missing");

		let mut content = String::new();
		self.write_prefix(
			&mut content,
			&attrs.metadata().into(),
			&PrefixOptions {
				align: false,
				#[cfg(feature = "timestamps")]
				time: None,
			},
		);
		let prefix_end_index = content.len();
		attrs.record(&mut FieldVisitor::new(&mut content));

		let mut extensions = span.extensions_mut();
		extensions.insert(SpanData {
			content,
			prefix_end_index,
		});
	}

	fn on_record(&self, id: &Id, values: &Record, ctx: Context<S>) {
		let span = ctx.span(id).expect("span missing");
		let mut extensions = span.extensions_mut();
		let data: &mut SpanData = extensions
			.get_mut()
			.expect("span missing SpanData extension");
		values.record(&mut FieldVisitor::new(&mut data.content));
	}

	fn on_event(&self, event: &Event, ctx: Context<S>) {
		#[cfg(feature = "timestamps")]
		let time = SystemTime::now();

		with_local_string(move |mut buf| {
			buf.clear();

			self.write_prefix(
				&mut buf,
				&event.metadata().into(),
				&PrefixOptions {
					align: true,
					#[cfg(feature = "timestamps")]
					time: Some(time),
				},
			);

			let mut i_buf = Indented::new(&mut buf, 8);
			event.record(&mut FieldVisitor::new(&mut i_buf));

			if let Some(parent_span) = ctx.event_span(event) {
				for span in parent_span.scope() {
					let extensions = span.extensions();
					let data: &SpanData =
						extensions.get().expect("span missing SpanData extension");
					let (prefix, fields) = data.content.split_at(data.prefix_end_index);
					i_buf.indent -= 2;
					i_buf.push('\n');
					i_buf.push_str(prefix);
					i_buf.indent += 2;

					let name = span.name();
					if !name.is_empty() {
						i_buf.push('\n');
						i_buf.push_str(name);
					}
					i_buf.push_str(fields);
				}
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

struct FieldVisitor<T: StringLike + fmt::Write>(T);

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
		self.0.push_str(zmij::Buffer::new().format(value));
	}
}
