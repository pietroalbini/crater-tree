extern crate cargo_metadata;
#[macro_use]
extern crate failure;
extern crate petgraph;
extern crate rayon;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tempdir;

mod crater_results;
mod cargo;
mod graph;
mod prelude;

use prelude::*;
use rayon::prelude::*;
use std::env;
use std::sync::Mutex;

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
    let graph = Mutex::new(graph::DependencyGraph::new());
    crates
        .par_iter()
        .map(|krate| -> Result<_> {
            let metadata = cargo::get_metadata(krate)?;
            if let Some(ref resolve) = metadata.resolve {
                graph.lock().unwrap().load_from_metadata(resolve);
            }

            Ok(())
        })
        .collect::<Result<()>>()?;
    let graph = graph.into_inner().unwrap();

    graph.display();

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
