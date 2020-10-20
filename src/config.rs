use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

use crate::target::Target;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub build_dir: String,
    pub default_target: String,
    pub targets: HashMap<String, Target>,
}
