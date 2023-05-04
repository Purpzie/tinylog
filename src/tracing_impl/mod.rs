mod visitor;

use self::visitor::FieldVisitor;
use crate::{
	util::{with_local_buf, Indented, StringLike},
	Logger, PrefixOptions,
};
use std::io;
use tracing::{
	span::{Attributes, Record},
	Event, Id, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

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

		with_local_buf(move |mut buf| {
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
