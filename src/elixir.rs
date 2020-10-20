use serde_derive::{Deserialize, Serialize};

use crate::target::{Dependencies, Dependency, LanguageTarget, Shell};

const BUILD_TEMPLATE: &'static str = include_str!("./templates/elixir/mix.exs.tmpl");
const SOURCE_TEMPLATE: &'static str = include_str!("./templates/elixir/parser.ex.tmpl");
const SHELL_TEMPLATE: &'static str = include_str!("./templates/elixir/shell.sh.tmpl");

#[derive(Deserialize, Serialize)]
pub struct Context {
    pub applications: Vec<String>,
    pub deps: Dependencies,
    pub dep_string: String,
}

pub fn new(deps: Dependencies) -> LanguageTarget<Context> {
    LanguageTarget::new(
        ("mix.exs", BUILD_TEMPLATE),
        Context {
            applications: generate_applications(deps.clone()),
            dep_string: generate_dep_string(deps.clone()),
            deps,
        },
        ("mix", vec!["do", "deps.get,", "deps.compile"]),
        Some(Shell::new(("iex", vec!["-S", "mix"]), SHELL_TEMPLATE)),
        "lib",
        vec![("parser.ex", SOURCE_TEMPLATE)],
    )
}

fn generate_dep_string(deps: Dependencies) -> String {
    deps.iter()
        .map(|Dependency { name, version }| format!("{{:{}, \"~> {}\"}}", name, version))
        .collect::<Vec<String>>()
        .join(", ")
}

fn generate_applications(deps: Dependencies) -> Vec<String> {
    deps.iter().map(|dep| dep.name.clone()).collect()
}
