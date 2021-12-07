use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

use crate::target::{
    DefaultTarget, Dependency, ProgramCommand, RemoteTarget, SupportedLanguage, Target,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub build_dir: String,
    pub cache: Option<bool>,
    pub default_target: String,
    pub targets: HashMap<String, Target>,
}

pub fn default() -> Config {
    let mut targets = HashMap::new();

    let elixir_deps = vec![
        Dependency::new("httpoison", "1.7"),
        Dependency::new("jason", "1.2"),
        Dependency::new("nimble_csv", "1.1"),
        Dependency::new("floki", "0.29"),
    ];
    let elixir_target =
        DefaultTarget::new("elixir".to_string(), SupportedLanguage::elixir, elixir_deps);
    targets.insert(
        SupportedLanguage::elixir.to_string(),
        Target::Internal(elixir_target),
    );

    let node_deps = vec![
        Dependency::new("axios", "0.20.0"),
        Dependency::new("cheerio", "1.0.0-rc.3"),
        Dependency::new("papaparse", "5.3.0"),
    ];
    let node_target = DefaultTarget::new("node".to_string(), SupportedLanguage::node, node_deps);
    targets.insert(
        SupportedLanguage::node.to_string(),
        Target::Internal(node_target),
    );

    let rust_deps = vec![
        Dependency::new("clap", "1.0"),
        Dependency::new("tokio", "0.9"),
    ];
    let rust_target = DefaultTarget::new("rust".to_string(), SupportedLanguage::rust, rust_deps);
    targets.insert(
        SupportedLanguage::rust.to_string(),
        Target::Internal(rust_target),
    );

    let serve_react_git = Target::Repo(RemoteTarget::new(
        String::from("https://github.com/rawhat/serve-react.git"),
        String::from("npm"),
        vec![String::from("install")],
        Some(ProgramCommand::new(String::from("./serve.sh"), vec![])),
    ));

    targets.insert("react".to_string(), serve_react_git);

    let phoenix_react_git = Target::Repo(RemoteTarget::new(
        String::from("https://github.com/rawhat/phoenix-react.git"),
        String::from("docker-compose"),
        vec![String::from("build")],
        Some(ProgramCommand::new(
            String::from("docker-compose"),
            vec![String::from("up")],
        )),
    ));
    targets.insert("phoenix_react".to_string(), phoenix_react_git);

    Config {
        build_dir: String::from("/tmp/shelly"),
        cache: Some(true),
        default_target: String::from("elixir"),
        targets,
    }
}
