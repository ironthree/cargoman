use serde::{Deserialize, Serialize};

use indexmap::IndexMap;

// inspired by: https://gitlab.com/crates.rs/cargo_toml

pub type Dependencies = IndexMap<String, Dependency>;

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
