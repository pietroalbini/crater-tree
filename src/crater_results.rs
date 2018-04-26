use prelude::*;
use reqwest;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

static REPORTS_BASE: &'static str = "https://cargobomb-reports.s3.amazonaws.com";

#[derive(Deserialize)]
pub enum Crate {
    Registry { name: String, version: String },
    GitHub { org: String, name: String },
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
    let url = format!("{}/{}/{}", REPORTS_BASE, ex, file);
    Ok(reqwest::get(&url)?.json()?)
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
