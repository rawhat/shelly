use std::fs;
use std::path;

use clap::Clap;

use shelly::config::{Config, SupportedLanguage, Target};
use shelly::elixir;

/// Generate dynamic, scripting language projects with dependencies for
/// quick CLI feedback loops.
#[derive(Clap)]
#[clap(version = "1.0", author = "Alex M. <alex41290@gmail.com>")]
struct Opts {
    /// Path to create project
    #[clap(default_value = ".")]
    path: String,

    /// Drop into REPL after building
    #[clap(short, long)]
    shell: bool,

    /// A target is a language and dependencies pairing
    #[clap(short, long)]
    target: Option<String>,

    #[clap(short, long)]
    config: Option<String>,
}

const DEFAULT_CONFIG: &'static str = include_str!("./templates/shelly.yml");

fn main() {
    let opts: Opts = Opts::parse();

    let config_file = if let Some(cfg) = opts.config {
        fs::read_to_string(cfg)
    } else {
        let home_path = path::PathBuf::from(std::env::var("HOME").unwrap());
        let path = home_path.join(".config").join("shelly");

        if path.clone().join("shelly.yml").as_path().exists() {
            fs::read_to_string(path.clone().join("shelly.yml"))
        } else {
            let _ = fs::create_dir_all(path.clone());
            let _ = fs::write(path.join("shelly.yml"), DEFAULT_CONFIG);
            Ok(String::from(DEFAULT_CONFIG))
        }
    }
    .unwrap();

    let config: Config = serde_yaml::from_str(config_file.as_str()).unwrap();

    let target_name = if let Some(t) = opts.target {
        t
    } else {
        config.default_target
    };
    let target = config.targets.get(&target_name).unwrap();
    let Target { language, deps } = target;

    match language {
        SupportedLanguage::elixir => {
            elixir::write_project(opts.path, deps.clone()).unwrap();
            elixir::run(opts.shell).unwrap();
        }
        SupportedLanguage::node => {
            panic!("Language not supported yet");
        }
        SupportedLanguage::rust => {
            panic!("Language not supported yet");
        }
    }
}
