# run

```toml
[scripts]
dev = "cargo watch -x run"
install = "cargo install --path ."
```

if you've used npm, it'll feel familiar.

## installation

```bash
cargo install --git https://github.com/manen/run
```

## usage

`run <script>`

create a `run.toml` file, and fill in your scripts like in the example. the directory you put the `run.toml` file in is the root of your workspace, all your commands will be executed from there.

the scripts are executed using bash, to allow for more complex operations (writing to files, piping, etc). when running your script, every argument after your script's name will be appended to the original script.
