fn main() -> anyhow::Result<()> {
	let mut args = std::env::args();
	let _arg0 = args.next().unwrap();

	run::run(args)
}
