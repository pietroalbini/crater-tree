use cargo_metadata::{self, Metadata};
use crater_results::Crate;
use prelude::*;
use std::fs::OpenOptions;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempdir::TempDir;

fn generate_dummy_project(krate: &Crate) -> Result<PathBuf> {
    let tmp = TempDir::new("crater-tree")?.into_path();
    let krate_name = format!("dummy-{}", krate.name());

    // Create a dummy cargo project
    let status = Command::new("cargo")
        .arg("new")
        .arg("--bin")
        .arg(&krate_name)
        .current_dir(&tmp)
        .status()?;
    ensure!(status.success(), "failed to create a cargo project");

    // Generate dependencies
    let deps = match *krate {
        Crate::Registry { ref name, .. } => {
            format!("{} = \"*\"\n", name)
        },
        Crate::GitHub { .. } => unreachable!(),
    };

    OpenOptions::new()
        .append(true)
        .open(tmp.join(&krate_name).join("Cargo.toml"))?
        .write_all(deps.as_bytes())?;

    Ok(tmp)
}

pub fn get_metadata(krate: &Crate) -> Result<Metadata> {
    let path = generate_dummy_project(&krate)
        .context("failed to generate dummy project")?;

    let metadata = cargo_metadata::metadata_deps(
        Some(&path.join(format!("dummy-{}", krate.name())).join("Cargo.toml")),
        true
    ).map_err(::failure::SyncFailure::new).context("failed to collect metadata")?;

    fs::remove_dir_all(&path).context("failed to remove dummy project")?;
    Ok(metadata)
}
