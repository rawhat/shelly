use std::env;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "linux")]
use std::os::unix::prelude::*;

use anyhow::anyhow;
use serde::ser::Serialize as SerdeSerialize;
use serde_derive::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tinytemplate::TinyTemplate;

use crate::opts::Opts;
use crate::{elixir, node};

/// The list of supported languages
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_camel_case_types)]
// TODO:  There is probably a supported way to map this to a capital
// letter, but I'm not bothering with that right now.
pub enum SupportedLanguage {
    elixir,
    node,
    rust,
}

impl fmt::Display for SupportedLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: &'static str = match self {
            SupportedLanguage::elixir => "elixir",
            SupportedLanguage::node => "node",
            SupportedLanguage::rust => "rust",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

pub type Dependencies = Vec<Dependency>;

pub type LanguageTemplate = (&'static str, &'static str);

pub type ProgramCommand = (&'static str, Vec<String>);

type GetShellArgs = Box<dyn Fn() -> anyhow::Result<ProgramCommand>>;

pub struct Shell {
    get_command: GetShellArgs,
    template: &'static str,
}

impl Shell {
    pub fn new(get_command: GetShellArgs, template: &'static str) -> Shell {
        Shell {
            get_command,
            template,
        }
    }
}

pub struct Template {
    path: String,
    template: String,
}

impl Template {
    pub fn new(path: String, template: String) -> Template {
        Template { path, template }
    }
}

pub struct Templates {
    build_template: Template,
    source_templates: Vec<Template>,
    shell_template: Option<Template>,
}

impl<'a> Templates {
    pub fn new(build_template: Template) -> Templates {
        Templates {
            build_template,
            source_templates: Vec::new(),
            shell_template: None,
        }
    }

    pub fn add_source_template(&mut self, path: String, template: String) {
        self.source_templates.push(Template { path, template });
    }

    pub fn add_shell_template(&mut self, template: String) {
        self.shell_template = Some(Template::new(String::from("shell.sh"), template));
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Target {
    pub language: SupportedLanguage,
    pub deps: Dependencies,
}

impl Target {
    pub fn execute(&self, opts: Opts) -> anyhow::Result<()> {
        match self.language {
            SupportedLanguage::elixir => {
                let target = elixir::new(self.deps.clone());
                target.write_project(opts.get_path())?;
                target.run()?;
                Ok(())
            }
            SupportedLanguage::node => {
                let target = node::new(self.deps.clone());
                target.write_project(opts.get_path())?;
                target.run()?;
                Ok(())
            }
            SupportedLanguage::rust => {
                Err(anyhow!("{} is not a supported language", self.language))
            }
        }
    }
}

trait GenerateHash {
    fn generate_hash(&self) -> String;
}

impl GenerateHash for Target {
    fn generate_hash(&self) -> String {
        let deps_string = self
            .deps
            .iter()
            .map(|dep| format!("{},{}", dep.name, dep.version))
            .collect::<Vec<String>>()
            .join(":");

        let mut hasher = Sha1::new();
        hasher.update(format!("{}{}", self.language, deps_string));
        String::from_utf8(hasher.finalize().into_iter().collect::<Vec<u8>>()).unwrap()
    }
}

pub struct LanguageTarget<T>
where
    T: SerdeSerialize,
{
    build_template: LanguageTemplate,
    context: T,
    run_command: ProgramCommand,
    shell: Option<Shell>,
    source_directory: &'static str,
    source_templates: Vec<LanguageTemplate>,
}

impl<T> LanguageTarget<T>
where
    T: SerdeSerialize,
{
    pub fn new(
        build_template: LanguageTemplate,
        context: T,
        run_command: ProgramCommand,
        shell: Option<Shell>,
        source_directory: &'static str,
        source_templates: Vec<LanguageTemplate>,
    ) -> LanguageTarget<T> {
        LanguageTarget {
            build_template,
            context,
            run_command,
            shell,
            source_directory,
            source_templates,
        }
    }

    fn generate_templates(&self) -> anyhow::Result<Templates> {
        let mut template = TinyTemplate::new();

        template.add_template(self.build_template.0, self.build_template.1)?;

        for (name, template_string) in self.source_templates.iter() {
            template.add_template(name, template_string)?;
        }

        if let Some(shell) = &self.shell {
            template.add_template("shell.sh", shell.template)?;
        }

        template.set_default_formatter(&tinytemplate::format_unescaped);

        let build_template = template
            .render(self.build_template.0, &self.context)
            .map_err(|err| anyhow!("Failed to render {}: {}", self.build_template.0, err))?;

        let mut templates = Templates::new(Template::new(
            String::from(self.build_template.0),
            build_template,
        ));

        for (name, _template) in self.source_templates.iter() {
            let source_template = template
                .render(name, &self.context)
                .map_err(|err| anyhow!("Failed to render source {}: {}", name, err))?;
            let source_path: String = PathBuf::from(self.source_directory)
                .join(name)
                .to_str()
                .unwrap()
                .into();
            templates.add_source_template(source_path, source_template);
        }

        if let Some(_) = self.shell {
            let shell_template = template
                .render("shell.sh", &self.context)
                .map_err(|err| anyhow!("Failed to render shell template: {}", err))?;
            templates.add_shell_template(shell_template);
        }

        Ok(templates)
    }

    pub fn write_project(&self, path: String) -> anyhow::Result<()> {
        println!("Generating project...");

        let folder_path = PathBuf::from(path);

        let templates = self.generate_templates()?;

        fs::create_dir_all(folder_path.clone())
            .map_err(|err| anyhow!("Failed to create project folder: {}", err))?;
        env::set_current_dir(folder_path.clone())
            .map_err(|err| anyhow!("Failed to change to project directory: {}", err))?;
        fs::write(
            templates.build_template.path,
            templates.build_template.template,
        )
        .map_err(|err| anyhow!("Failed to write build template: {}", err))?;
        fs::create_dir(PathBuf::from(self.source_directory))
            .map_err(|err| anyhow!("Failed to create `lib` directory: {}", err))?;

        for Template { path, template } in templates.source_templates.iter() {
            fs::write(path, template)
                .map_err(|err| anyhow!("Failed to write source file: {}", err))?;
        }

        if let Some(shell) = &templates.shell_template {
            fs::write(shell.path.clone(), shell.template.clone())
                .map_err(|err| anyhow!("Failed to write shell template: {}", err))?;
            let mut permissions = fs::metadata(shell.path.clone())?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(shell.path.clone(), permissions)?;
        }

        Ok(())
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let stdout = os_pipe::dup_stdout()?;
        let mut cmd = Command::new(self.run_command.0)
            .args(self.run_command.1.clone())
            .stdout(stdout)
            .spawn()
            .map_err(|err| anyhow!("Failed to spawn {} command: {}", self.run_command.0, err))?;

        let _ = cmd.wait();

        if let Some(shell) = &self.shell {
            let stdout = os_pipe::dup_stdout()?;
            let shell_args = (shell.get_command)()?;
            let mut shell_cmd = Command::new(shell_args.0)
                .args(shell_args.1.clone())
                .stdout(stdout)
                .stdin(os_pipe::dup_stdin().unwrap())
                .spawn()
                .map_err(|err| anyhow!("Failed to spawn {} command: {}", shell_args.0, err))?;

            let _ = shell_cmd.wait();
        }

        Ok(())
    }
}
