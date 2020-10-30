use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::eval::is_linux_target;

// inspired by: https://gitlab.com/crates.rs/cargo_toml

pub type Dependencies = IndexMap<String, Dependency>;

// Non-public struct fields are present to preserve the order
// of sections and values in rewritten Cargo.toml files.
// The goal is to make only minimal changes to Cargo.toml files
// that were normalized before uploading them to crates.io.

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    package: toml::Value,
    profile: Option<toml::Value>,
    lib: Option<toml::Value>,
    bin: Option<toml::Value>,
    example: Option<toml::Value>,
    test: Option<toml::Value>,
    bench: Option<toml::Value>,
    pub dependencies: Option<Dependencies>,
    pub dev_dependencies: Option<Dependencies>,
    pub build_dependencies: Option<Dependencies>,
    pub features: Option<IndexMap<String, Vec<String>>>,
    pub target: Option<IndexMap<String, Target>>,
    badges: Option<toml::Value>,
    #[serde(flatten)]
    remainder: IndexMap<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Target {
    pub dependencies: Option<Dependencies>,
    pub dev_dependencies: Option<Dependencies>,
    pub build_dependencies: Option<Dependencies>,
    #[serde(flatten)]
    remainder: IndexMap<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Version(String),
    Details(DependencyDetails),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyDetails {
    pub version: Option<String>,
    pub features: Option<Vec<String>>,
    pub optional: Option<bool>,
    pub default_features: Option<bool>,
    pub package: Option<String>,
    #[serde(flatten)]
    remainder: IndexMap<String, toml::Value>,
}

impl Manifest {
    // returned boolean indicates whether the manifest was changed or not
    pub fn normalize_targets(&mut self) -> Result<bool, String> {
        if let Some(ref mut target) = self.target {
            for (key, target) in target.drain(..) {
                if is_linux_target(&key)? {
                    if let Some(mut dependencies) = target.dependencies {
                        for (key, dep) in dependencies.drain(..) {
                            if let Some(ref mut dependencies) = self.dependencies {
                                dependencies.insert(key, dep);
                            } else {
                                let mut index: Dependencies = IndexMap::new();
                                index.insert(key, dep);
                                self.dependencies = Some(index);
                            }
                        }
                    }

                    if let Some(mut dev_dependencies) = target.dev_dependencies {
                        for (key, dep) in dev_dependencies.drain(..) {
                            if let Some(ref mut dev_dependencies) = self.dev_dependencies {
                                dev_dependencies.insert(key, dep);
                            } else {
                                let mut index: Dependencies = IndexMap::new();
                                index.insert(key, dep);
                                self.dev_dependencies = Some(index);
                            }
                        }
                    }

                    if let Some(mut build_dependencies) = target.build_dependencies {
                        for (key, dep) in build_dependencies.drain(..) {
                            if let Some(ref mut build_dependencies) = self.build_dependencies {
                                build_dependencies.insert(key, dep);
                            } else {
                                let mut index: Dependencies = IndexMap::new();
                                index.insert(key, dep);
                                self.build_dependencies = Some(index);
                            }
                        }
                    }
                }
            }

            self.target = None;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn set_dependency_version(&mut self, dependency: &str, version: &str) -> Result<(), String> {
        let doit = |deps: &mut Dependencies| {
            deps.entry(dependency.to_string()).and_modify(|value| match value {
                Dependency::Version(ref mut s) => {
                    // modify string in-place
                    s.clear();
                    s.push_str(version);
                },
                Dependency::Details(ref mut details) => {
                    details.version = Some(version.to_string());
                },
            });
        };

        if let Some(ref mut deps) = self.dependencies {
            doit(deps);
        }

        if let Some(ref mut deps) = self.dev_dependencies {
            doit(deps);
        }

        if let Some(ref mut deps) = self.build_dependencies {
            doit(deps);
        }

        Ok(())
    }

    pub fn remove_feature(&mut self, feature: &str) -> Result<(), String> {
        let features = match self.features {
            Some(ref mut features) => features,
            None => return Err(String::from("No features present in Cargo.toml.")),
        };

        // keep track of removed features and optional dependencies
        let mut removed: Vec<String> = Vec::new();

        match features.shift_remove(feature) {
            Some(values) => removed.extend(values),
            None => return Err(format!("Feature not present in Cargo.toml: {}", feature)),
        }

        // remove removed feature from other feature's dependencies
        for (_name, dependencies) in features.iter_mut() {
            dependencies.retain(|d| d != feature);
        }

        // remove optional dependencies that are no longer necessary
        let mut optionals: Vec<String> = Vec::new();
        if let Some(ref deps) = self.dependencies {
            for (name, details) in deps {
                if let Dependency::Details(details) = details {
                    if details.optional.unwrap_or(false) {
                        optionals.push(name.to_string());
                    }
                }
            }
        }

        for dropped in &removed {
            if optionals.contains(dropped) {
                // only remove dropped optional dependencies if no remaining feature depends on them
                let mut dep_of_feature = false;
                for (_name, deps) in features.iter() {
                    if deps.contains(dropped) {
                        dep_of_feature = true;
                    }
                }

                if dep_of_feature {
                    continue;
                }

                if let Some(ref mut deps) = self.dependencies {
                    deps.shift_remove(dropped);
                }
            }
        }

        Ok(())
    }
}
