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
    pub name: String,
    pub version: String,
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

// TODO:  move this OUT of here... it should be on a per-language basis
#[derive(Deserialize, Serialize)]
pub struct Context {
    pub deps: String,
}
