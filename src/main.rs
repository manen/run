use anyhow::anyhow;
use serde::Deserialize;
use std::{
	collections::HashMap,
	env, fs,
	io::{self, Write},
	path::PathBuf,
	process::{Command, Stdio},
};

macro_rules! quit {
	($($arg:tt)*) => {{
		eprintln!($($arg)*);
		::std::process::exit(1);
	}};
}
macro_rules! quit_unwrap {
	($result:expr, $msg:expr) => {{
		match $result {
			Ok(a) => a,
			Err(a) => {
				let debug = if cfg!(debug_assertions) {
					format!("{a}\n")
				} else {
					String::new()
				};
				quit!("{debug}{}", $msg);
			}
		}
	}};
	($result:expr) => {{
		match $result {
			Ok(a) => a,
			Err(a) => {
				quit!("{a}");
			}
		}
	}};
	(opt $opt:expr, $msg:expr) => {{
		match $opt {
			Some(a) => a,
			None => quit!("{}", $msg),
		}
	}};
}

#[derive(Deserialize)]
struct Config {
	scripts: HashMap<String, String>,
}

fn config(mut dir: PathBuf) -> anyhow::Result<(Config, PathBuf)> {
	if dir.ancestors().count() == 1 {
		return Err(anyhow!("couldn't find a run.toml file"));
	}
	let path = dir.join("run.toml");
	let text = match fs::read_to_string(&path) {
		Ok(a) => a,
		Err(a) => match a.kind() {
			io::ErrorKind::NotFound => {
				dir.pop();
				return config(dir);
			}
			_ => return Err(a.into()),
		},
	};

	let cfg: Config = toml::from_str(&text)?;
	Ok((cfg, dir))
}

fn main() {
	let (cfg, cwd) = quit_unwrap!(config(env::current_dir().unwrap()));

	// parse args
	let mut args = env::args();
	let arg0 = args.next().unwrap();
	let script = match args.next() {
		None => quit!("usage: {arg0} <script>"),
		Some(a) => a,
	};
	let pass_args = args.collect::<Vec<_>>().join(" ");

	// read command from run.toml, append passed args
	let cmd = match cfg.scripts.get(&script) {
		None => quit!("no script called {script} in run.toml"),
		Some(a) => a,
	};
	let cmd = format!("{cmd} {pass_args}");

	// launch child process
	let mut child = Command::new(quit_unwrap!(opt
		pathsearch::find_executable_in_path("bash"),
		"install bash to use run"
	))
	.stdin(Stdio::piped())
	.stdout(io::stdout())
	.current_dir(&cwd)
	.spawn()
	.unwrap();
	{
		let child_stdin = child.stdin.as_mut().unwrap();
		child_stdin.write_all(cmd.as_bytes()).unwrap();
	}
	child.wait().unwrap();
}
