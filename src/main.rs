// Copyright (c) 2018 Pietro Albini <pietro@pietroalbini.org>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is furnished to do
// so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

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
