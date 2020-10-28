use std::fs;
use std::io::Write;

use indexmap::IndexMap;
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

// returned boolean indicates whether the manifest was changed or not
fn normalize_targets(manifest: &mut Manifest) -> Result<bool, String> {
    if let Some(ref mut target) = manifest.target {
        for (key, target) in target.drain(..) {
            if is_linux_target(&key)? {
                if let Some(mut dependencies) = target.dependencies {
                    for (key, dep) in dependencies.drain(..) {
                        if let Some(ref mut dependencies) = manifest.dependencies {
                            dependencies.insert(key, dep);
                        } else {
                            let mut index: Dependencies = IndexMap::new();
                            index.insert(key, dep);
                            manifest.dependencies = Some(index);
                        }
                    }
                }

                if let Some(mut dev_dependencies) = target.dev_dependencies {
                    for (key, dep) in dev_dependencies.drain(..) {
                        if let Some(ref mut dev_dependencies) = manifest.dev_dependencies {
                            dev_dependencies.insert(key, dep);
                        } else {
                            let mut index: Dependencies = IndexMap::new();
                            index.insert(key, dep);
                            manifest.dev_dependencies = Some(index);
                        }
                    }
                }

                if let Some(mut build_dependencies) = target.build_dependencies {
                    for (key, dep) in build_dependencies.drain(..) {
                        if let Some(ref mut build_dependencies) = manifest.build_dependencies {
                            build_dependencies.insert(key, dep);
                        } else {
                            let mut index: Dependencies = IndexMap::new();
                            index.insert(key, dep);
                            manifest.build_dependencies = Some(index);
                        }
                    }
                }
            }
        }

        manifest.target = None;

        Ok(true)
    } else {
        Ok(false)
    }
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

    if let Some(ref mut deps) = manifest.dependencies {
        doit(deps);
    }

    if let Some(ref mut deps) = manifest.dev_dependencies {
        doit(deps);
    }

    if let Some(ref mut deps) = manifest.build_dependencies {
        doit(deps);
    }

    Ok(())
}

fn main() -> Result<(), String> {
    env_logger::init();

    let arguments: Arguments = Arguments::from_args();

    let result: Result<(), String> = match arguments {
        Arguments::NormalizeTargets { ref path } => {
            let (mut manifest, preamble) = read_manifest(path)?;

            // only write back to disk if there actually are changes
            let rewritten = normalize_targets(&mut manifest)?;
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
            set_dependency_version(&mut manifest, dependency, version)?;
            write_manifest(path, &manifest, &preamble)?;
            Ok(())
        },
    };

    result
}
