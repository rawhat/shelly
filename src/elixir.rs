use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Result};
use tinytemplate::TinyTemplate;

use crate::config::{Context, Dependencies, Dependency};

const BUILD_TEMPLATE: &'static str = include_str!("./templates/elixir/mix.exs.tmpl");
const SOURCE_TEMPLATE: &'static str = include_str!("./templates/elixir/parser.ex.tmpl");

fn generate_templates(deps: Dependencies) -> Result<Vec<(&'static str, String)>> {
    let mut template = TinyTemplate::new();
    template.add_template("mix.exs", BUILD_TEMPLATE)?;
    template.add_template("parser.ex", SOURCE_TEMPLATE)?;
    template.set_default_formatter(&tinytemplate::format_unescaped);

    let dep_string = deps
        .iter()
        .map(|Dependency { name, version }| format!("{{:{}, \"~> {}\"}}", name, version))
        .collect::<Vec<String>>()
        .join(", ");

    let build_template = template
        .render(
            "mix.exs",
            &Context {
                deps: dep_string.clone(),
            },
        )
        .map_err(|err| anyhow!("Failed to render mix.exs template: {}", err))?;

    let source_template = template
        .render("parser.ex", &Context { deps: dep_string })
        .map_err(|err| anyhow!("Failed to render parser.ex template: {}", err))?;

    let mut templates = Vec::new();

    templates.push(("mix.exs", build_template));
    templates.push(("lib/parser.ex", source_template));

    Ok(templates)
}

pub fn write_project(path: String, deps: Dependencies) -> Result<()> {
    println!("Generating mix project...");

    let folder_path = PathBuf::from(path);

    let templates = generate_templates(deps.clone())?;

    let (build_path, build_template) = templates.get(0).unwrap();
    let (source_path, source_template) = templates.get(1).unwrap();

    fs::create_dir_all(folder_path.clone())
        .map_err(|err| anyhow!("Failed to create project folder: {}", err))?;
    env::set_current_dir(folder_path.clone())
        .map_err(|err| anyhow!("Failed to change to project directory: {}", err))?;
    fs::write(build_path, build_template)
        .map_err(|err| anyhow!("Failed to write build template: {}", err))?;
    fs::create_dir(PathBuf::from("lib"))
        .map_err(|err| anyhow!("Failed to create `lib` directory: {}", err))?;
    fs::write(source_path, source_template)
        .map_err(|err| anyhow!("Failed to write source file: {}", err))?;

    Ok(())
}

pub fn run(shell: bool) -> Result<()> {
    let stdout = os_pipe::dup_stdout()?;
    let mut mix = Command::new("mix")
        .args(&["do", "deps.get,", "deps.compile"])
        .stdout(stdout)
        .spawn()
        .map_err(|err| anyhow!("Failed to spawn mix command: {}", err))?;

    let _ = mix.wait();

    if shell {
        let stdout = os_pipe::dup_stdout()?;
        let mut iex_command = Command::new("iex")
            .args(&["-S", "mix"])
            .stdout(stdout)
            .stdin(os_pipe::dup_stdin().unwrap())
            .spawn()
            .map_err(|err| anyhow!("Failed to spawn iex command: {}", err))?;

        let _ = iex_command.wait();
    }

    Ok(())
}
