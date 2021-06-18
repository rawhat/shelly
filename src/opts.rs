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

    #[clap(long)]
    no_cache: bool,
}

impl Opts {
    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn get_shell(&self) -> bool {
        self.shell
    }

    pub fn get_target(&self) -> Option<String> {
        self.target.clone()
        /* match &self.target {
        Some(t) => Some(t.clone()),
        None => None, */
        // }
    }

    pub fn get_config(&self) -> Option<String> {
        self.config.clone()
    }

    pub fn get_no_cache(&self) -> bool {
        self.no_cache
    }
}
