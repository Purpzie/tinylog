use anyhow::{bail, ensure, Context};
use log::{debug, error, info, trace, warn};
use std::{env, fmt::Write, process};

type Result<T = ()> = anyhow::Result<T>;

/// Run a test.
///
/// The name *must* be the same as the name of the test function.
fn run_test<R, C>(test_name: &str, run: R, check: C) -> Result
where
	R: FnOnce() -> Result,
	C: FnOnce(&str) -> Result,
{
	if env::var("TESTING_TESTING_TESTING").is_ok() {
		return run();
	}

	let (_, this_module) = module_path!()
		.split_once("::")
		.context("invalid module path")?;

	#[rustfmt::skip]
	let result = process::Command::new("cargo").arg("test")
		.args([
			"--color", "never",
			"--quiet",
			"--lib",
			"--",
			&format!("{}::{}", this_module, test_name),
			"--exact",
			"--nocapture",
		])
		.env("TESTING_TESTING_TESTING", "1")
		.env("TERM", "dumb") // termcolor respects this
		.env("RUST_LOG", "trace")
		.output()
		.context("could not start child process")?;

	let stdout = String::from_utf8(result.stdout).context("invalid utf8 in stdout")?;

	if !result.status.success() {
		let stderr = String::from_utf8(result.stderr).context("invalid utf8 in stderr")?;
		bail!("child process failed\n{}\n{}", stdout, stderr);
	}

	match check(&stdout) {
		Ok(()) => Ok(()),
		Err(err) => {
			bail!("{:?}\n{}", err, stdout);
		}
	}
}

macro_rules! log_all {
	($($args:tt)*) => {
		error!($($args)*);
		warn!($($args)*);
		info!($($args)*);
		debug!($($args)*);
		trace!($($args)*);
	}
}

const LEVEL_NAMES: [&str; 5] = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
const TERM_COLOR_ON: &str = "xterm-256color";

#[test]
fn force_color() -> Result {
	run_test(
		"force_color",
		|| {
			env::set_var("FORCE_COLOR", "1");
			crate::init();
			log_all!("test");
			Ok(())
		},
		|output| {
			ensure!(output.contains('\x1b'));
			Ok(())
		},
	)
}

#[test]
fn no_color() -> Result {
	run_test(
		"no_color",
		|| {
			env::set_var("NO_COLOR", "1");
			env::set_var("TERM", TERM_COLOR_ON);
			crate::init();
			log_all!("test");
			Ok(())
		},
		|output| {
			ensure!(!output.contains('\x1b'));
			Ok(())
		},
	)
}

#[test]
fn set_color_1() -> Result {
	run_test(
		"set_color_1",
		|| {
			crate::config().force_color().init();
			log_all!("test");
			Ok(())
		},
		|output| {
			ensure!(output.contains('\x1b'));
			Ok(())
		},
	)
}

#[test]
fn set_color_2() -> Result {
	run_test(
		"set_color_2",
		|| {
			env::set_var("TERM", TERM_COLOR_ON);
			crate::config().no_color().init();
			log_all!("test");
			Ok(())
		},
		|output| {
			ensure!(!output.contains('\x1b'));
			Ok(())
		},
	)
}

#[test]
fn filter() -> Result {
	run_test(
		"filter",
		|| {
			crate::config().filter(|m| m.target() != "bad").init();
			log_all!(target: "bad", "if you see this, the test failed");
			log_all!(target: "good", "this is fine");
			Ok(())
		},
		|output| {
			ensure!(!output.contains("if you see this, the test failed"));
			for level in LEVEL_NAMES {
				ensure!(output.contains(&format!("{} (good) this is fine", level)));
			}
			Ok(())
		},
	)
}

#[test]
fn display_level() -> Result {
	use std::string::ToString;

	run_test(
		"display_level",
		|| {
			crate::config()
				.display_level(|r, f| {
					let mut level = r.level().to_string();
					level.make_ascii_lowercase();
					f.write_str(&level)
				})
				.init();
			log_all!(target: "test", "test");
			Ok(())
		},
		|output| {
			for level in LEVEL_NAMES.into_iter().map(|n| n.to_ascii_lowercase()) {
				ensure!(output.contains(&format!("{} (test) test", level)));
			}
			Ok(())
		},
	)
}

#[test]
fn display_target() -> Result {
	run_test(
		"display_target",
		|| {
			crate::config()
				.display_target(|r, f| write!(f, " foo [{}] ", r.target().to_uppercase()))
				.init();
			log_all!(target: "target_name", "test");
			Ok(())
		},
		|output| {
			for level in LEVEL_NAMES {
				ensure!(output.contains(&format!("{} foo [TARGET_NAME] test", level)));
			}
			Ok(())
		},
	)
}

#[test]
fn display_content() -> Result {
	run_test(
		"display_content",
		|| {
			crate::config()
				.display_content(|r, f| {
					let mut content = format!("{}", r.args());
					content.make_ascii_uppercase();
					f.write_str(&content)
				})
				.init();

			log_all!(target: "test", "should be uppercase");
			Ok(())
		},
		|output| {
			for level in LEVEL_NAMES {
				ensure!(output.contains(&format!("{} (test) SHOULD BE UPPERCASE", level)));
			}
			Ok(())
		},
	)
}
