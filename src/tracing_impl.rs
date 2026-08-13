use crate::{
	Logger, PrefixOptions,
	util::{Indented, MutexHelper, StringLike, with_local_string},
};
#[cfg(feature = "timestamps")]
use std::time::SystemTime;
use std::{fmt, io};
use tracing::{
	Event, Id, Subscriber,
	field::{Field, Visit},
	span::{Attributes, Record},
};
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};

struct SpanData {
	content: String,
	prefix_end_index: usize,
}

impl<S, T> Layer<S> for Logger<T>
where
	T: io::Write + 'static,
	S: Subscriber + for<'any> LookupSpan<'any>,
{
	fn on_new_span(&self, attrs: &Attributes, id: &Id, ctx: Context<S>) {
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
		attrs.record(&mut FieldVisitor(&mut content));

		ctx.span(id)
			.expect("span missing")
			.extensions_mut()
			.insert(SpanData {
				content,
				prefix_end_index,
			});
	}

	fn on_record(&self, id: &Id, values: &Record, ctx: Context<S>) {
		values.record(&mut FieldVisitor(
			&mut ctx
				.span(id)
				.expect("span missing")
				.extensions_mut()
				.get_mut::<SpanData>()
				.expect("span missing SpanData extension")
				.content,
		));
	}

	fn on_event(&self, event: &Event, ctx: Context<S>) {
		#[cfg(feature = "timestamps")]
		let time = SystemTime::now();

		with_local_string(move |mut buf| {
			self.write_prefix(
				&mut buf,
				&event.metadata().into(),
				&PrefixOptions {
					align: true,
					#[cfg(feature = "timestamps")]
					time: Some(time),
				},
			);

			let mut i_buf = Indented::new(&mut buf, Self::PREFIX_LABEL_LEN);
			event.record(&mut FieldVisitor(&mut i_buf));

			for span in ctx.event_scope(event).into_iter().flatten() {
				let extensions = span.extensions();
				let data: &SpanData = extensions.get().expect("span missing SpanData extension");
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

			buf.push('\n');
			self.output
				.lock_ignore_poison()
				.write_all(buf.as_bytes())
				.expect("io error");
		})
	}
}

impl<S, T> Layer<S> for &'static Logger<T>
where
	T: io::Write + 'static,
	S: Subscriber + for<'any> LookupSpan<'any>,
{
	fn on_new_span(&self, attrs: &Attributes, id: &Id, ctx: Context<S>) {
		(*self).on_new_span(attrs, id, ctx)
	}

	fn on_record(&self, id: &Id, values: &Record, ctx: Context<S>) {
		(*self).on_record(id, values, ctx)
	}

	fn on_event(&self, event: &Event, ctx: Context<S>) {
		(*self).on_event(event, ctx)
	}
}

struct FieldVisitor<T>(T);

impl<T> FieldVisitor<T>
where
	T: StringLike,
{
	fn write_field(&mut self, field: &Field) {
		self.0.push('\n');
		let name = field.name();
		if name != "message" {
			self.0.push_str(name);
			self.0.push_str(": ");
		}
	}
}

impl<T> Visit for FieldVisitor<T>
where
	T: StringLike + fmt::Write,
{
	fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
		self.write_field(field);
		write!(self.0, "{value:?}").expect("fmt error");
	}

	fn record_str(&mut self, field: &Field, value: &str) {
		self.write_field(field);
		if field.name() == "message" {
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
