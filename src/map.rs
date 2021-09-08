use std::fmt::{self, Display};

pub(super) struct DisplayMap<'a>(
	pub &'a dyn Fn(&str, &mut fmt::Formatter) -> fmt::Result,
	pub &'a str,
);

impl Display for DisplayMap<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0(self.1, f)
	}
}

#[test]
#[cfg(test)]
fn display_map() {
	let display_map = DisplayMap(&|_, f| f.write_str("correct"), "nope");
	assert_eq!(format!("{}", display_map), "correct");
}
