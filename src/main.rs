extern crate cargo_metadata;
#[macro_use]
extern crate failure;
extern crate rayon;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tempdir;

mod crater_results;
mod cargo;
mod prelude;

use prelude::*;
use rayon::prelude::*;
use std::env;

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

    println!("Collecting crates metadata...");
    let metadata = crates.par_iter()
        .map(|krate| -> Result<_> {
            Ok((krate.clone(), cargo::get_metadata(krate)?))
        })
        .collect::<Result<Vec<_>>>()?;
    println!("Collected metadata of {} crates", metadata.len());

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
