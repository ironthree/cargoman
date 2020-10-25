use std::fs;
use std::io::Write;

use structopt::StructOpt;

mod eval;
use eval::is_linux_target;

mod manifest;
use manifest::{Dependencies, Dependency, Manifest};

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

    write!(file, "\
# This file has been changed by cargoman.
#
{}

{}", preamble, string).map_err(|err| format!("Failed to write file: {}", err))?;

    Ok(())
}

fn normalize_targets(manifest: &mut Manifest) -> Result<(), String> {
    for (key, mut target) in manifest.target.drain(..) {
        if is_linux_target(&key)? {
            for (key, dep) in target.dependencies.drain(..) {
                manifest.dependencies.insert(key, dep);
            }
            for (key, dep) in target.dev_dependencies.drain(..) {
                manifest.dev_dependencies.insert(key, dep);
            }
            for (key, dep) in target.build_dependencies.drain(..) {
                manifest.build_dependencies.insert(key, dep);
            }
        }
    }

    Ok(())
}

fn set_dependency_version(manifest: &mut Manifest, dependency: &str, version: &str) -> Result<(), String> {
    let doit = |deps: &mut Dependencies| {
        deps.entry(dependency.to_string()).and_modify(|value| match value {
            Dependency::Version(ref mut s) => {
                s.clear();
                s.push_str(version);
            },
            Dependency::Details(ref mut details) => {
                details.version = Some(version.to_string());
            },
        });
    };

    doit(&mut manifest.dependencies);
    doit(&mut manifest.dev_dependencies);
    doit(&mut manifest.build_dependencies);

    Ok(())
}

fn main() -> Result<(), String> {
    env_logger::init();

    let arguments: Arguments = Arguments::from_args();

    let result: Result<(), String> = match arguments {
        Arguments::NormalizeTargets { ref path } => {
            let (mut manifest, preamble) = read_manifest(path)?;
            normalize_targets(&mut manifest)?;
            write_manifest(path, &manifest, &preamble)?;
            Ok(())
        },
        Arguments::SetDependencyVersion {
            ref path,
            ref dependency,
            ref version,
        } => {
            let (mut manifest, preamble) = read_manifest(path)?;
            set_dependency_version(&mut manifest, dependency, version)?;
            write_manifest(path, &manifest, &preamble)?;
            Ok(())
        },
    };

    result
}
