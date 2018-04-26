#[macro_use]
extern crate failure;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod crater_results;
mod prelude;

use std::env;
use prelude::*;

fn run() -> Result<()> {
    // Simple argument parsing
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        bail!("usage: crater-tree <experiment-name>");
    }
    let experiment_name = args[0].as_str();

    println!("Loading list of regressed crates...");
    let crates = crater_results::load_regressed(experiment_name)
        .context("failed to load results from crater")?;
    println!("Loaded {} regressions.", crates.len());

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {}", error);
        for cause in error.causes().skip(1) {
            eprintln!("  caused by: {}", cause);
        }
        ::std::process::exit(1);
    }
}
