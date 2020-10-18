use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Result, anyhow};
use tinytemplate::TinyTemplate;

use crate::config::{Context, Dependencies, Dependency};

const BUILD_TEMPLATE: &'static str = include_str!("./templates/node/package.json.tmpl");
const SOURCE_TEMPLATE: &'static str = include_str!("./templates/node/index.js.tmpl");

fn generate_templates(deps: Dependencies) -> Result<Vec<(&'static str, String)>> {
    let mut template = TinyTemplate::new();
    template.add_template("package.json", BUILD_TEMPLATE)?;
    template.add_template("index.js", SOURCE_TEMPLATE)?;
    template.set_default_formatter(&tinytemplate::format_unescaped);

    let dep_string: String = deps.iter().map(|Dependency{name, version}| {
        format!("\"{}\": \"^{}\"", name, version)
    })
    .collect::<Vec<String>>()
    .join(",\n");

    let build_template = template
        .render("package.json", &Context { deps: dep_string.clone() })
        .map_err(|err| anyhow!("Failed to render package.json: {}", err))?;

    let source_template = template
        .render("index.js", &Context { deps: dep_string })
        .map_err(|err| anyhow!("Failed to render index.js: {}", err))?;

    let mut templates = Vec::new();

    templates.push(("package.json", build_template));
    templates.push(("src/index.js", source_template));

    Ok(templates)
}

pub fn write_project(path: String, deps: Dependencies) -> Result<()> {
    println!("Generating node project...");

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
    fs::create_dir(PathBuf::from("src"))
        .map_err(|err| anyhow!("Failed to create `lib` directory: {}", err))?;
    fs::write(source_path, source_template)
        .map_err(|err| anyhow!("Failed to write source file: {}", err))?;

    Ok(())
}

pub fn run(shell: bool) -> Result<()> {
    let stdout = os_pipe::dup_stdout()?;
    let mut npm = Command::new("npm")
        .arg("i")
        .stdout(stdout)
        .spawn()
        .map_err(|err| anyhow!("Failed to spawn npm command: {}", err))?;

    let _ = npm.wait();

    if shell {
        let stdout = os_pipe::dup_stdout()?;
        let index_js = fs::read_to_string("./src/index.js")?;
        let mut node_cmd = Command::new("node")
            .args(&["-i", "--experimental-repl-await", "-e", index_js.as_str()])
            .stdout(stdout)
            .stdin(os_pipe::dup_stdin().unwrap())
            .spawn()
            .map_err(|err| anyhow!("Failed to spawn node command: {}", err))?;

        let _ = node_cmd.wait();
    }

    Ok(())
}
