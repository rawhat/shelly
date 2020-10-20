use std::fs;
use std::path;

use anyhow::{anyhow, Result};
use clap::Clap;

use shelly::config::Config;
use shelly::opts::Opts;

const DEFAULT_CONFIG: &'static str = include_str!("./templates/shelly.yml");

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let config_file = if let Some(cfg) = opts.get_config() {
        fs::read_to_string(cfg)?
    } else {
        let home_path = path::PathBuf::from(
            std::env::var("HOME").map_err(|err| anyhow!("Failed to read $HOME: {}", err))?,
        );
        let path = home_path.join(".config").join("shelly");

        if path.clone().join("shelly.yml").as_path().exists() {
            fs::read_to_string(path.clone().join("shelly.yml"))
                .map_err(|err| anyhow!("Failed to read existing `shelly.yml`: {}", err))?
        } else {
            fs::create_dir_all(path.clone())
                .map_err(|err| anyhow!("Failed to create config directory: {}", err))?;
            fs::write(path.join("shelly.yml"), DEFAULT_CONFIG)
                .map_err(|err| anyhow!("Failed to write default config file: {}", err))?;
            String::from(DEFAULT_CONFIG)
        }
    };

    let config: Config = serde_yaml::from_str(config_file.as_str())
        .map_err(|err| anyhow!("Error parsing config file: {}", err))?;

    let target_name = if let Some(t) = opts.get_target() {
        t
    } else {
        config.default_target
    };
    config
        .targets
        .get(&target_name)
        .ok_or(anyhow!("Target not specified in `shelly.yml` file"))?
        .execute(opts)
}
