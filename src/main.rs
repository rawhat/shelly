use std::fs;
use std::path;

use anyhow::{anyhow, Result};
use clap::Clap;

use shelly::config::Config;
use shelly::opts::Opts;
use shelly::target::{copy_build_directory, pull_git_repo, Target};

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let config_file = if let Some(cfg) = opts.get_config() {
        fs::read_to_string(cfg)?
    } else {
        let home_path = path::PathBuf::from(
            std::env::var("HOME").map_err(|err| anyhow!("Failed to read $HOME: {}", err))?,
        );
        let path = home_path.join(".config").join("shelly");

        if path.join("shelly.yml").as_path().exists() {
            fs::read_to_string(path.join("shelly.yml"))
                .map_err(|err| anyhow!("Failed to read existing `shelly.yml`: {}", err))?
        } else {
            let default = shelly::config::default();
            let default_config = serde_yaml::to_string(&default)?;
            fs::create_dir_all(path.clone())
                .map_err(|err| anyhow!("Failed to create config directory: {}", err))?;
            fs::write(path.join("shelly.yml"), default_config.clone())
                .map_err(|err| anyhow!("Failed to write default config file: {}", err))?;
            default_config
        }
    };

    let config: Config = serde_yaml::from_str(config_file.as_str())
        .map_err(|err| anyhow!("Error parsing config file: {}", err))?;

    let target_name = opts.get_target().unwrap_or(config.default_target);

    let target = config
        .targets
        .get(&target_name)
        .ok_or_else(|| anyhow!("Target not specified in `shelly.yml` file"))?;

    match target {
        Target::Internal(t) => t.execute(opts, config.build_dir),
        Target::Repo(repo) => pull_git_repo(opts.get_path(), repo, opts.get_shell()),
        Target::Directory(dir) => copy_build_directory(opts.get_path(), dir, opts.get_shell()),
    }
}
