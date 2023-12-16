use serde::Deserialize;
use std::{
	collections::HashMap,
	env, fs,
	io::{self, Write},
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

fn main() {
	let mut cwd = env::current_dir().unwrap();
	cwd.push("run.toml");

	// read run.toml
	let file: String = quit_unwrap!(fs::read_to_string(cwd), "couldn't read run.toml");
	let cfg: Config = quit_unwrap!(toml::from_str(&file), "couldn't parse run.toml");

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
	let mut child = Command::new(quit_unwrap!(opt
		pathsearch::find_executable_in_path("bash"),
		"install bash to use run"
	))
	.stdin(Stdio::piped())
	.stdout(io::stdout())
	.spawn()
	.unwrap();
	{
		let child_stdin = child.stdin.as_mut().unwrap();
		child_stdin.write_all(cmd.as_bytes()).unwrap();
	}
	child.wait().unwrap();
}
