use clap::Clap;

/// Generate dynamic, scripting language projects with dependencies for
/// quick CLI feedback loops.
#[derive(Clap)]
#[clap(version = "1.0", author = "Alex M. <alex41290@gmail.com>")]
pub struct Opts {
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

impl Opts {
    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn get_shell(&self) -> bool {
        self.shell
    }

    pub fn get_target(&self) -> Option<String> {
        match &self.target {
            Some(t) => Some(t.clone()),
            None => None,
        }
    }

    pub fn get_config(&self) -> Option<String> {
        match &self.config {
            Some(cfg) => Some(cfg.clone()),
            None => None,
        }
    }
}
