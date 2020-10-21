use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

use crate::target::{Dependency, SupportedLanguage, Target};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub build_dir: String,
    pub default_target: String,
    pub targets: HashMap<String, Target>,
}

pub fn default() -> Config {
    let mut targets = HashMap::new();

    let mut elixir_deps = Vec::new();
    elixir_deps.push(Dependency::new("httpoison", "1.7"));
    elixir_deps.push(Dependency::new("jason", "1.2"));
    elixir_deps.push(Dependency::new("nimble_csv", "1.1"));
    elixir_deps.push(Dependency::new("floki", "0.29"));
    let elixir_target = Target::new(SupportedLanguage::elixir, elixir_deps);
    targets.insert(SupportedLanguage::elixir.to_string(), elixir_target);

    let mut node_deps = Vec::new();
    node_deps.push(Dependency::new("axios", "0.20.0"));
    node_deps.push(Dependency::new("cheerio", "1.0.0-rc.3"));
    node_deps.push(Dependency::new("papaparse", "5.3.0"));
    let node_target = Target::new(SupportedLanguage::node, node_deps);
    targets.insert(SupportedLanguage::node.to_string(), node_target);

    let mut rust_deps = Vec::new();
    rust_deps.push(Dependency::new("clap", "1.0"));
    rust_deps.push(Dependency::new("tokio", "0.9"));
    let rust_target = Target::new(SupportedLanguage::rust, rust_deps);
    targets.insert(SupportedLanguage::rust.to_string(), rust_target);

    Config{
        build_dir: String::from("/tmp/shelly"),
        default_target: String::from("elixir"),
        targets,
    }
}
