use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tinytemplate::TinyTemplate;

use crate::config::{Context, Dependencies};

fn build_template() -> String {
    fs::read_to_string(
        PathBuf::from("src")
            .join("templates")
            .join("elixir")
            .join("mix.exs.tmpl"),
    )
    .unwrap()
}

fn source_template() -> String {
    fs::read_to_string(
        PathBuf::from("src")
            .join("templates")
            .join("elixir")
            .join("parser.ex.tmpl"),
    )
    .unwrap()
}

fn generate_templates(deps: Dependencies) -> Vec<(&'static str, String)> {
    let build = build_template();
    let source = source_template();

    let mut template = TinyTemplate::new();
    template.add_template("mix.exs", &build).unwrap();
    template.add_template("parser.ex", &source).unwrap();

    let build_template = template
        .render("mix.exs", &Context { deps: deps.clone() })
        .unwrap();

    let source_template = template
        .render("parser.ex", &Context { deps })
        .unwrap();

    let mut templates = Vec::new();

    templates.push(("mix.exs", build_template));
    templates.push(("lib/parser.ex", source_template));

    templates
}

pub fn write_project(path: String, deps: Dependencies) -> Result<(), String> {
    println!("Generating mix project...");

    let folder_path = PathBuf::from(path);

    let templates = generate_templates(deps.clone());

    let (build_path, build_template) = templates.get(0).unwrap();
    let (source_path, source_template) = templates.get(1).unwrap();

    fs::create_dir_all(folder_path.clone())
        .map_err(|err| format!("Failed to create dir: {}", err))
        .and_then(|_| {
            env::set_current_dir(folder_path.clone())
                .map_err(|err| format!("Failed to set cwd: {}", err))
        })
        .and_then(|_| {
            fs::write(build_path, build_template)
                .map_err(|err| format!("Failed to write `mix.exs`: {}", err))
        })
        .and_then(|_| {
            fs::create_dir(PathBuf::from("lib"))
                .map_err(|err| format!("Failed to create `lib` dir: {}", err))
        })
        .and_then(|_| {
            fs::write(source_path, source_template)
                .map_err(|err| format!("Failed to write `lib/parser.ex`: {}", err))
        })
}

pub fn run(shell: bool) -> Result<(), String> {
    let stdout = os_pipe::dup_stdout().unwrap();
    let mut mix = match Command::new("mix")
        .args(&["do", "deps.get,", "deps.compile"])
        .stdout(stdout)
        .spawn()
    {
        Err(error) => panic!("Failed to spawn `mix` process: {}", error),
        Ok(process) => process,
    };

    let _ = mix.wait();

    if shell {
        let stdout = os_pipe::dup_stdout().unwrap();
        let mut iex_command = match Command::new("iex")
            .args(&["-S", "mix"])
            .stdout(stdout)
            .stdin(os_pipe::dup_stdin().unwrap())
            .spawn()
        {
            Err(error) => panic!("Failed to run `iex` shell: {}", error),
            Ok(process) => process,
        };

        let _ = iex_command.wait();
    }

    Ok(())
}
