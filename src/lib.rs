use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::{
	collections::HashMap,
	env, fs,
	io::{self},
	path::PathBuf,
};

#[derive(Debug, Clone, Deserialize)]
/// the run.toml file is like a tree; it has branches, that is how you define subcommands
struct Config {
	#[serde(flatten)]
	cfg: HashMap<String, Entry>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Entry {
	Command(String),
	Config(Config),
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
fn cfg_search<A: Iterator<Item = String>>(cfg: &Config, mut args: A) -> Result<&str> {
	let arg = args.next().unwrap_or_else(|| "index".to_owned());
	let entry = cfg
		.cfg
		.get(&arg)
		.ok_or_else(|| anyhow!("couldn't find {arg} in config ({cfg:?})"))?;

	match entry {
		Entry::Command(cmd) => Ok(cmd.as_str()),
		Entry::Config(child_cfg) => cfg_search(child_cfg, args),
	}
}

pub fn run<A: IntoIterator<Item = String>>(args: A) -> anyhow::Result<()> {
	let args = args.into_iter();
	let (cfg, cwd) = config(env::current_dir().unwrap())?;

	let capture_io = std::env::var("RUN_CAPTURE_IO")
		.map(|s| match s.as_ref() {
			"0" => false,
			_ => true,
		})
		.unwrap_or_default();

	let cmd = cfg_search(&cfg, args)?;

	if capture_io {
		let out = bash::capture_io::run_with_cwd(&cmd, &cwd)?;
		println!("{out}");
	} else {
		bash::inherit::run_with_cwd(&cmd, &cwd)?;
	};
	Ok(())
}
