use std::path::PathBuf;

use clap::Parser;

mod core;
mod prelude;
#[cfg(test)]
mod tests;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    cwd: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    let cwd = cli
        .cwd
        .unwrap_or(std::env::current_dir().expect("current dir is always available"));

    match core::run(&cwd) {
        Err(err) => eprintln!("{:?}", miette::Report::new(err)),
        _ => {}
    }
}
