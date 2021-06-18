use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "linux")]
use std::os::unix::prelude::*;

use anyhow::anyhow;
use serde::ser::Serialize as SerdeSerialize;
use serde_derive::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tinytemplate::TinyTemplate;
use walkdir::{DirEntry, WalkDir};

use crate::opts::Opts;
use crate::{elixir, node};

/// The list of supported languages
#[derive(Clone, Debug, Deserialize, Serialize)]
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

impl Dependency {
    pub fn new(name: &'static str, version: &'static str) -> Dependency {
        Dependency {
            name: String::from(name),
            version: String::from(version),
        }
    }
}

pub type Dependencies = Vec<Dependency>;

pub type LanguageTemplate = (&'static str, &'static str);

#[derive(Debug, Deserialize, Serialize)]
pub struct ProgramCommand {
    command: String,
    args: Vec<String>,
}

impl ProgramCommand {
    pub fn new(command: String, args: Vec<String>) -> ProgramCommand {
        ProgramCommand { command, args }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let mut cmd = self.get_command()?;
        let mut child = cmd.spawn()?;
        let _ = child.wait()?;
        Ok(())
    }

    pub fn run_with_stdin(&self) -> anyhow::Result<()> {
        let mut cmd = self.get_command()?;
        cmd.stdin(os_pipe::dup_stdin()?);
        let mut child = cmd.spawn()?;
        let _ = child.wait()?;
        Ok(())
    }

    fn get_command(&self) -> anyhow::Result<Command> {
        let stdout = os_pipe::dup_stdout()?;
        let mut cmd = Command::new(self.command.clone());
        cmd.args(self.args.clone());
        cmd.stdout(stdout);

        Ok(cmd)
    }
}

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
pub struct DefaultTarget {
    pub language: Option<SupportedLanguage>,
    pub name: String,
    pub deps: Option<Dependencies>,
}

impl DefaultTarget {
    pub fn new(name: String, language: SupportedLanguage, deps: Dependencies) -> DefaultTarget {
        DefaultTarget {
            deps: Some(deps),
            language: Some(language),
            name,
        }
    }

    pub fn execute(&self, opts: Opts, build_dir: String) -> anyhow::Result<()> {
        match self.language {
            Some(SupportedLanguage::elixir) => {
                let target = elixir::new(self.deps.clone().unwrap(), opts.get_shell());
                let cached = target.is_cached(self.name.clone(), build_dir);
                match (opts.get_no_cache(), cached) {
                    (true, _) | (false, false) => {
                        target.write_project(opts.get_path())?;
                        // target.write_hash(self.name.clone(), build_dir)?;
                        target.run()?;
                        Ok(())
                    }
                    _ => {
                        // copy from hashed entry!
                        Ok(())
                    }
                }
            }
            Some(SupportedLanguage::node) => {
                let target = node::new(self.deps.clone().unwrap(), opts.get_shell());
                let cached = target.is_cached(self.name.clone(), build_dir);
                match (opts.get_no_cache(), cached) {
                    (true, _) | (false, false) => {
                        target.write_project(opts.get_path())?;
                        // target.write_hash(self.name.clone(), build_dir)?;
                        target.run()?;
                        Ok(())
                    }
                    _ => {
                        // copy from hashed entry!
                        Ok(())
                    }
                }
            }
            _ => Err(anyhow!(
                "{} is not a supported language",
                self.language.clone().unwrap()
            )),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemoteTarget {
    build_args: Vec<String>,
    build_command: String,
    path: String,
    shell: Option<ProgramCommand>,
}

impl RemoteTarget {
    pub fn new(
        path: String,
        build_command: String,
        build_args: Vec<String>,
        shell: Option<ProgramCommand>,
    ) -> RemoteTarget {
        RemoteTarget {
            build_args,
            build_command,
            path,
            shell,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Target {
    Internal(DefaultTarget),
    Directory(RemoteTarget),
    Repo(RemoteTarget),
}

pub fn generate_hash(deps: Dependencies, language: SupportedLanguage) -> String {
    let dep_string = deps
        .iter()
        .map(|dep| format!("{},{}", dep.name, dep.version))
        .collect::<Vec<String>>()
        .join(":");

    let mut hasher = Sha256::new();
    let bytes = [
        language.to_string().as_str().as_bytes(),
        dep_string.as_bytes(),
    ]
    .concat();
    hasher.update(&bytes);
    format!("{:X}", hasher.finalize())
}

pub struct LanguageTarget<T>
where
    T: SerdeSerialize,
{
    build_template: LanguageTemplate,
    context: T,
    hash: String,
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
        hash: String,
        run_command: ProgramCommand,
        shell: Option<Shell>,
        source_directory: &'static str,
        source_templates: Vec<LanguageTemplate>,
    ) -> LanguageTarget<T> {
        LanguageTarget {
            build_template,
            context,
            hash,
            run_command,
            shell,
            source_directory,
            source_templates,
        }
    }

    fn hash_path(&self, name: String, build_dir: String) -> String {
        let file_name = format!("{}.sha1", name);
        String::from(
            Path::new(build_dir.as_str())
                .join(file_name.as_str())
                .to_str()
                .expect("Bad sha1 file name generated"),
        )
    }

    fn is_cached(&self, name: String, build_dir: String) -> bool {
        let hash_path = self.hash_path(name, build_dir);
        fs::read_to_string(hash_path).map_or(false, |hash| hash == self.hash);
        false
    }

    /* fn write_hash(&self, name: String, build_dir: String) -> anyhow::Result<()> {
        let hash_path = self.hash_path(name, build_dir);
        fs::write(hash_path, self.hash.clone())
            .map_err(|err| anyhow!("Failed to write hash: {}", err))
    } */

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

        if self.shell.is_some() {
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
        env::set_current_dir(folder_path)
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
        self.run_command.run().map_err(|err| {
            anyhow!(
                "Failed to spawn {} command: {}",
                self.run_command.command,
                err
            )
        })?;

        if let Some(shell) = &self.shell {
            (shell.get_command)()?.run()?;
        }

        Ok(())
    }
}

pub fn pull_git_repo(project_path: String, repo: &RemoteTarget, shell: bool) -> anyhow::Result<()> {
    if shell && repo.shell.is_none() {
        return Err(anyhow!(
            "No shell command specified in config for this git repo"
        ));
    }

    ProgramCommand::new(
        String::from("git"),
        vec![
            String::from("clone"),
            repo.path.clone(),
            project_path.clone(),
        ],
    )
    .run()
    .map_err(|err| anyhow!("Failed to clone git repo {}: {}", repo.path.clone(), err))?;

    env::set_current_dir(project_path)
        .map_err(|err| anyhow!("Failed to change to project directory: {}", err))?;

    ProgramCommand::new(repo.build_command.clone(), repo.build_args.clone())
        .run()
        .map_err(|err| {
            anyhow!(
                "Failed to run build command for git repo {}: {}",
                repo.path,
                err
            )
        })?;

    if shell {
        repo.shell
            .as_ref()
            .unwrap()
            .run_with_stdin()
            .map_err(|err| anyhow!("Failed to run build shell command: {}", err))?;
    }

    Ok(())
}

pub fn copy_build_directory(
    project_path: String,
    build: &RemoteTarget,
    shell: bool,
) -> anyhow::Result<()> {
    if shell && build.shell.is_none() {
        return Err(anyhow!(
            "No shell command specified in config for this build folder"
        ));
    }

    fs::create_dir_all(project_path.clone())
        .map_err(|err| anyhow!("Failed to create project folder: {}", err))?;

    let ignored_dirs = vec!["node_modules"];

    let files: Vec<DirEntry> = WalkDir::new(build.path.clone())
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let file = entry
                .path()
                .strip_prefix(build.path.clone())
                .unwrap()
                .to_str()
                .unwrap();
            !ignored_dirs.iter().any(|dir| file.starts_with(dir))
        })
        .collect();

    for file in files {
        println!("file path: {:?}", file.path());
        let folder_path = file.path().strip_prefix(build.path.clone())?;
        let path_to_file = PathBuf::from(project_path.clone()).join(folder_path);
        if file.metadata()?.is_dir() {
            println!("Creating folder {:?}", file.path());
            fs::create_dir_all(file.path())?;
        } else {
            println!("Writing {:?} to {:?}", file.path(), path_to_file);
            fs::copy(file.path(), path_to_file)?;
        }
    }

    env::set_current_dir(project_path)
        .map_err(|err| anyhow!("Failed to change to project directory: {}", err))?;

    ProgramCommand::new(build.build_command.clone(), build.build_args.clone())
        .run()
        .map_err(|err| {
            anyhow!(
                "Failed to run build command for build directory {}: {}",
                build.path,
                err
            )
        })?;

    if shell {
        build
            .shell
            .as_ref()
            .unwrap()
            .run_with_stdin()
            .map_err(|err| anyhow!("Failed to run git shell command: {}", err))?;
    }

    Ok(())
}
