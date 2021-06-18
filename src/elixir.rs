use serde_derive::{Deserialize, Serialize};

use crate::target::{
    generate_hash, Dependencies, Dependency, LanguageTarget, ProgramCommand, Shell,
    SupportedLanguage,
};

const BUILD_TEMPLATE: &str = include_str!("./templates/elixir/mix.exs.tmpl");
const SOURCE_TEMPLATE: &str = include_str!("./templates/elixir/parser.ex.tmpl");
const SHELL_TEMPLATE: &str = include_str!("./templates/elixir/shell.sh.tmpl");

#[derive(Deserialize, Serialize)]
pub struct Context {
    pub applications: Vec<String>,
    pub deps: Dependencies,
    pub dep_string: String,
}

pub fn new(deps: Dependencies, shell: bool) -> LanguageTarget<Context> {
    LanguageTarget::new(
        ("mix.exs", BUILD_TEMPLATE),
        Context {
            applications: generate_applications(deps.clone()),
            dep_string: generate_dep_string(deps.clone()),
            deps: deps.clone(),
        },
        generate_hash(deps, SupportedLanguage::elixir),
        ProgramCommand::new(
            String::from("mix"),
            vec![
                "do".to_string(),
                "deps.get,".to_string(),
                "deps.compile".to_string(),
            ],
        ),
        if shell {
            Some(Shell::new(
                Box::new(|| {
                    Ok(ProgramCommand::new(
                        String::from("iex"),
                        vec!["-S".to_string(), "mix".to_string()],
                    ))
                }),
                SHELL_TEMPLATE,
            ))
        } else {
            None
        },
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
