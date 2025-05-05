use std::{
	env,
	io::{self, Write},
	path::Path,
	process::{Command, Stdio},
	string::FromUtf8Error,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("failed to run {cmd}: exit code {code}\n{stderr}")]
	FailedToRun {
		cmd: String,
		code: i32,
		stderr: String,
	},
	#[error("exit code: {0}")]
	ExitCode(i32),
	#[error("bash wasn't found in path env variable")]
	NoBashInPath,
	#[error("io error: {0}")]
	Io(#[from] io::Error),
	#[error("child process doesn't have stdin")]
	NoStdin,
	#[error("child process doesn't have stdout")]
	NoStdout,
	#[error("child process doesn't have stderr")]
	NoStderr,
	#[error("child process doesn't have an exit code")]
	NoExitCode,
	#[error("fromutf8error: {0}")]
	FromUtf8Error(#[from] FromUtf8Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub mod capture_io {
	use super::*;

	pub fn run<S: AsRef<str>>(cmd: S) -> Result<String> {
		run_with_cwd(cmd, env::current_dir()?)
	}
	pub fn run_with_cwd(cmd: impl AsRef<str>, cwd: impl AsRef<Path>) -> Result<String> {
		let cmd = cmd.as_ref();
		let mut child =
			Command::new(pathsearch::find_executable_in_path("bash").ok_or(Error::NoBashInPath)?)
				.stdin(Stdio::piped())
				.stdout(Stdio::piped())
				.current_dir(cwd)
				.spawn()?;
		();
		{
			let child_stdin = child.stdin.as_mut().ok_or(Error::NoStdin)?;
			child_stdin.write_all(cmd.as_bytes())?
		}
		let out = child.wait_with_output()?;
		let stdout = String::from_utf8(out.stdout)?.trim().to_string();

		match out.status.code().ok_or(Error::NoExitCode)? {
			0 => Ok(stdout),
			code => Err(Error::FailedToRun {
				cmd: cmd.to_owned(),
				code,
				stderr: String::from_utf8(out.stderr)?,
			}),
		}
	}
}

pub mod inherit {
	use super::*;

	pub fn run<S: AsRef<str>>(cmd: S) -> Result<()> {
		run_with_cwd(cmd, env::current_dir()?)
	}
	pub fn run_with_cwd(cmd: impl AsRef<str>, cwd: impl AsRef<Path>) -> Result<()> {
		let cmd = cmd.as_ref();

		let cwd = cwd.as_ref();
		let cwd = cwd.canonicalize()?;

		let mut child = std::process::Command::new(
			pathsearch::find_executable_in_path("bash").ok_or(Error::NoBashInPath)?,
		)
		.stdin(Stdio::piped())
		.current_dir(cwd)
		.spawn()?;

		// pipe the command into stdin
		{
			let child_stdin = child.stdin.as_mut().ok_or(Error::NoStdin)?;
			child_stdin.write_all(cmd.as_bytes())?
		}

		let status = child.wait()?;
		match status.code() {
			Some(0) => Ok(()),
			Some(code) => Err(Error::ExitCode(code)),
			None => Err(Error::NoExitCode),
		}?;

		Ok(())
	}
}
