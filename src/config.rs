use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

/// The list of supported languages
#[derive(Debug, Deserialize, Serialize)]
pub enum SupportedLanguage {
    elixir,
    node,
    rust,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dependency {
    name: String,
    version: String,
}

pub type Dependencies = Vec<Dependency>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Target {
    pub language: SupportedLanguage,
    pub deps: Dependencies,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub default_target: String,
    pub targets: HashMap<String, Target>,
}

#[derive(Deserialize, Serialize)]
pub struct Context {
    pub deps: Dependencies,
}
