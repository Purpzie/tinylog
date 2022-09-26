use std::{cell::RefCell, fmt};

pub(super) fn with_local_buf<F, R>(f: F) -> R
where
	F: FnOnce(&mut String) -> R,
{
	thread_local! {
		static BUF: RefCell<String> = RefCell::new(String::new());
	}

	let mut f = Some(f);
	BUF.try_with(|ref_cell| {
		ref_cell
			.try_borrow_mut()
			.ok()
			.map(|mut s| f.take().unwrap()(&mut s))
	})
	.ok()
	.flatten()
	.unwrap_or_else(|| f.take().unwrap()(&mut String::default()))
}

/// Similar to [`std::fmt::Write`], but with infallible methods.
pub(super) trait StringLike {
	fn push(&mut self, c: char);
	fn push_str(&mut self, s: &str);
	fn reserve(&mut self, additional: usize);
}

impl<T: StringLike> StringLike for &mut T {
	fn push(&mut self, c: char) {
		(**self).push(c);
	}

	fn push_str(&mut self, s: &str) {
		(**self).push_str(s);
	}

	fn reserve(&mut self, additional: usize) {
		(**self).reserve(additional);
	}
}

impl StringLike for String {
	fn push(&mut self, c: char) {
		self.push(c);
	}

	fn push_str(&mut self, s: &str) {
		self.push_str(s);
	}

	fn reserve(&mut self, additional: usize) {
		self.reserve(additional);
	}
}

/// Indents all text written to it by a certain amount.
#[non_exhaustive]
pub(super) struct Indented<T> {
	pub output: T,

	/// How many spaces to indent by.
	pub indent: usize,
}

impl<T> Indented<T> {
	pub fn new(output: T, indent: usize) -> Self {
		Self { output, indent }
	}
}

impl<T: StringLike> StringLike for Indented<T> {
	fn push(&mut self, c: char) {
		if c == '\n' {
			self.output.reserve(self.indent + 1);
			self.output.push('\n');
			for _ in 0..self.indent {
				self.output.push(' ');
			}
		} else {
			self.output.push(c);
		}
	}

	fn push_str(&mut self, s: &str) {
		let mut lines = s.split('\n');
		if let Some(first_line) = lines.next() {
			self.output.push_str(first_line);
			let indent = self.indent + 1;
			for line in lines {
				self.output.reserve(indent + line.len());
				self.output.push('\n');
				for _ in 0..self.indent {
					self.output.push(' ');
				}
				self.output.push_str(line);
			}
		}
	}

	fn reserve(&mut self, additional: usize) {
		self.output.reserve(additional);
	}
}

impl<T: StringLike> fmt::Write for Indented<T> {
	fn write_char(&mut self, c: char) -> std::fmt::Result {
		self.push(c);
		Ok(())
	}

	fn write_str(&mut self, s: &str) -> std::fmt::Result {
		self.push_str(s);
		Ok(())
	}
}
