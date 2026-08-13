use std::{cell::Cell, fmt};

/// Operate on an empty, reused, thread-local string. Falls back to a new one if it isn't available.
pub(crate) fn with_local_string<F, R>(f: F) -> R
where
	F: FnOnce(&mut String) -> R,
{
	thread_local! {
		static GLOBAL_STR: Cell<Option<String>> = const { Cell::new(Some(String::new())) };
	}

	if let Some(mut s) = GLOBAL_STR.try_with(Cell::take).ok().flatten() {
		s.clear();
		let result = f(&mut s);

		// If this fails, then our thread is shutting down. We can't do anything about it.
		// We shouldn't panic here because we might be inside of another panic.
		// Losing the string won't break anything.
		let _ = GLOBAL_STR.try_with(move |cell| cell.set(Some(s)));
		result
	} else {
		f(&mut String::new())
	}
}

/// Similar to [`std::fmt::Write`], but with infallible methods and no `dyn`.
pub(crate) trait StringLike {
	/// Append a single character.
	fn push(&mut self, c: char);
	/// Append a string slice.
	fn push_str(&mut self, s: &str);
	/// Reserve capacity for `additional` more bytes.
	fn reserve(&mut self, additional: usize);
}

impl<T> StringLike for &mut T
where
	T: StringLike,
{
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
pub(crate) struct Indented<T> {
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
			self.output.reserve(1 + self.indent);
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
			for line in lines {
				self.output.reserve(1 + self.indent + line.len());
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

impl<T> fmt::Write for Indented<T>
where
	T: StringLike,
{
	fn write_char(&mut self, c: char) -> std::fmt::Result {
		self.push(c);
		Ok(())
	}

	fn write_str(&mut self, s: &str) -> std::fmt::Result {
		self.push_str(s);
		Ok(())
	}
}

/// Helper trait to make it easier to switch between mutex types.
pub(crate) trait MutexHelper<'a> {
	type Output: 'a;
	fn lock_ignore_poison(&'a self) -> Self::Output;
}

impl<'a, T> MutexHelper<'a> for std::sync::Mutex<T>
where
	T: 'a,
{
	type Output = std::sync::MutexGuard<'a, T>;
	fn lock_ignore_poison(&'a self) -> Self::Output {
		self.lock().unwrap_or_else(|e| e.into_inner())
	}
}

#[cfg(feature = "parking_lot")]
impl<'a, T> MutexHelper<'a> for parking_lot::Mutex<T>
where
	T: 'a,
{
	type Output = parking_lot::MutexGuard<'a, T>;
	fn lock_ignore_poison(&'a self) -> Self::Output {
		self.lock()
	}
}
