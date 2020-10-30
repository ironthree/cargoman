use std::fs;
use std::io::Write;

use structopt::StructOpt;

mod eval;
mod manifest;

use manifest::Manifest;

#[derive(Debug, StructOpt)]
#[structopt(setting = structopt::clap::AppSettings::DisableHelpSubcommand)]
/// command line tool for doing basic programmatic manipulation of Cargo.toml files
enum Arguments {
    /// normalize targets (removes dependencies for foreign targets)
    NormalizeTargets {
        /// path to Cargo.toml file
        path: String,
    },
    /// override the version of a specific dependency
    SetDependencyVersion {
        /// path to Cargo.toml file
        path: String,
        /// name of the crate
        dependency: String,
        /// version override string
        version: String,
    },
}

fn read_manifest(path: &str) -> Result<(Manifest, String), String> {
    let string = fs::read_to_string(path).map_err(|err| format!("Failed to read Cargo.toml: {}", err))?;

    let manifest: Manifest =
        toml::from_str(&string).map_err(|err| format!("Failed to deserialize Cargo.toml: {}", err))?;

    let mut preamble: Vec<&str> = Vec::new();
    for line in string.lines() {
        if line.starts_with('#') {
            preamble.push(line);
        } else {
            break;
        }
    }

    Ok((manifest, preamble.join("\n")))
}

fn write_manifest(path: &str, manifest: &Manifest, preamble: &str) -> Result<(), String> {
    let string = toml::to_string(manifest).map_err(|err| format!("Failed to serialize manifest: {}", err))?;

    let mut file = fs::File::create(path).map_err(|err| format!("Failed to open Cargo.toml: {}", err))?;

    write!(
        file,
        "\
# This file has been changed by cargoman.
#
{}

{}",
        preamble, string
    )
    .map_err(|err| format!("Failed to write file: {}", err))?;

    Ok(())
}

fn main() -> Result<(), String> {
    env_logger::init();

    let arguments: Arguments = Arguments::from_args();

    let result: Result<(), String> = match arguments {
        Arguments::NormalizeTargets { ref path } => {
            let (mut manifest, preamble) = read_manifest(path)?;

            // only write back to disk if there actually are changes
            let rewritten = manifest.normalize_targets()?;
            if rewritten {
                write_manifest(path, &manifest, &preamble)?;
            }

            Ok(())
        },
        Arguments::SetDependencyVersion {
            ref path,
            ref dependency,
            ref version,
        } => {
            let (mut manifest, preamble) = read_manifest(path)?;
            manifest.set_dependency_version(dependency, version)?;
            write_manifest(path, &manifest, &preamble)?;
            Ok(())
        },
    };

    result
}
