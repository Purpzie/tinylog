mod visitor;

use self::visitor::FieldVisitor;
use crate::{
	util::{with_local_buf, Indented, StringLike},
	Logger, PrefixOptions,
};
use std::{io, time::Instant};
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
	timestamp: Instant,
}

impl<S, T: io::Write + Send + Sync + 'static> Layer<S> for Logger<T>
where
	S: Subscriber + for<'any> LookupSpan<'any>,
{
	fn on_new_span(&self, attrs: &Attributes, id: &Id, ctx: Context<S>) {
		let span = ctx.span(id).expect("span missing");
		let timestamp = Instant::now();

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
			timestamp,
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
		let event_timestamp = Instant::now();
		#[cfg(feature = "timestamps")]
		let time = SystemTime::now();
		let color = self.color;

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

					// if at least 1 millisecond has passed, let's print it
					let time_passed = event_timestamp - data.timestamp;
					const NANOS_PER_MILLISECOND: u32 = 1_000_000;
					if time_passed.subsec_nanos() >= NANOS_PER_MILLISECOND
						|| time_passed.as_secs() > 0
					{
						let seconds = time_passed.as_secs();
						let milliseconds = time_passed.subsec_millis();

						i_buf.push(' ');
						if color {
							// dim
							i_buf.push_str("\x1b[2m");
						}
						i_buf.push('(');
						if seconds == 0 {
							i_buf.push_str(itoa::Buffer::new().format(milliseconds));
							i_buf.push('m');
						} else {
							i_buf.push_str(
								ryu::Buffer::new()
									.format((seconds as f64) + ((milliseconds as f64) / 1000.0)),
							);
						}
						i_buf.push_str("s ago)");
						if color {
							// reset
							i_buf.push_str("\x1b[m");
						}
					}

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
