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

use prelude::*;
use reqwest;
use serde::de::DeserializeOwned;
use serde_json;
use std::collections::HashMap;
use std::fs::File;

static REPORTS_BASE: &'static str = "https://cargobomb-reports.s3.amazonaws.com";

#[derive(Clone, Deserialize)]
pub enum Crate {
    Registry { name: String, version: String },
    GitHub { org: String, name: String },
}

impl Crate {
    pub fn name(&self) -> String {
        match *self {
            Crate::Registry { ref name, .. } => name.clone(),
            Crate::GitHub { ref org, ref name } => format!("{}/{}", org, name),
        }
    }
}

#[derive(Deserialize)]
struct Config {
    crates: Vec<Crate>,
}

#[derive(Deserialize)]
struct Results {
    crates: Vec<CrateResult>,
}

#[derive(Deserialize)]
struct CrateResult {
    name: String,
    res: String,
}

fn load_file<T: DeserializeOwned>(ex: &str, file: &str) -> Result<T> {
    let path = format!("{}/{}", ex, file);
    Ok(match File::open(&path) {
        Ok(f) => serde_json::from_reader(f)?,
        Err(_e) => {
            let url = format!("{}/{}", REPORTS_BASE, path);
            reqwest::get(&url)?.json()?
        }
    })
}

pub fn load_regressed(ex: &str) -> Result<Vec<Crate>> {
    // config.json is used to get the structured crate details
    let config: Config = load_file(ex, "config.json")?;

    // Create an HashMap to quickly lookup structured data from the results
    let mut crates = HashMap::new();
    for krate in config.crates.into_iter() {
        let name = if let Crate::Registry {
            ref name,
            ref version,
        } = krate
        {
            format!("{}-{}", name, version)
        } else {
            continue;
        };

        crates.insert(name, krate);
    }

    // Collect all the regressed crates
    let mut regressed = Vec::new();
    let results: Results = load_file(ex, "results.json")?;
    for result in &results.crates {
        if result.res == "Regressed" {
            if let Some(krate) = crates.remove(&result.name) {
                regressed.push(krate);
            }
        }
    }

    Ok(regressed)
}
