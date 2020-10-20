use std::fs;

use serde_derive::{Deserialize, Serialize};

use crate::target::{Dependencies, Dependency, LanguageTarget, Shell};

const BUILD_TEMPLATE: &'static str = include_str!("./templates/node/package.json.tmpl");
const SOURCE_TEMPLATE: &'static str = include_str!("./templates/node/index.js.tmpl");
const SHELL_TEMPLATE: &'static str = include_str!("./templates/node/shell.sh.tmpl");

#[derive(Deserialize, Serialize)]
pub struct Context {
    pub deps: Dependencies,
    pub dep_string: String,
    pub packages: Vec<String>,
}

pub fn new(deps: Dependencies) -> LanguageTarget<Context> {
    LanguageTarget::new(
        ("package.json", BUILD_TEMPLATE),
        Context {
            deps: deps.clone(),
            dep_string: generate_dep_string(deps.clone()),
            packages: generate_packages(deps),
        },
        ("npm", vec!["i".to_string()]),
        Some(Shell::new(
            Box::new(|| {
                let source = fs::read_to_string("./src/index.js")?;
                Ok((
                    "node",
                    vec![
                        "-i".to_string(),
                        "--experimental-repl-await".to_string(),
                        "-e".to_string(),
                        source,
                    ],
                ))
            }),
            SHELL_TEMPLATE,
        )),
        "src",
        vec![("index.js", SOURCE_TEMPLATE)],
    )
}

fn generate_dep_string(deps: Dependencies) -> String {
    deps.iter()
        .map(|Dependency { name, version }| format!("\"{}\": \"^{}\"", name, version))
        .collect::<Vec<String>>()
        .join(",\n")
}

fn generate_packages(deps: Dependencies) -> Vec<String> {
    deps.iter().map(|dep| dep.name.clone()).collect()
}
