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

use cargo_metadata::{self, Metadata};
use crater_results::Crate;
use prelude::*;
use std::fs;
use std::fs::OpenOptions;
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
        Crate::Registry { ref name, .. } => format!("{} = \"*\"\n", name),
        Crate::GitHub { .. } => unreachable!(),
    };

    OpenOptions::new()
        .append(true)
        .open(tmp.join(&krate_name).join("Cargo.toml"))?
        .write_all(deps.as_bytes())?;

    Ok(tmp)
}

pub fn get_metadata(krate: &Crate) -> Result<Metadata> {
    let path = generate_dummy_project(&krate).context("failed to generate dummy project")?;

    let metadata = cargo_metadata::metadata_deps(
        Some(
            &path
                .join(format!("dummy-{}", krate.name()))
                .join("Cargo.toml"),
        ),
        true,
    )
    .map_err(::failure::SyncFailure::new)
    .context("failed to collect metadata")?;

    fs::remove_dir_all(&path).context("failed to remove dummy project")?;
    Ok(metadata)
}
