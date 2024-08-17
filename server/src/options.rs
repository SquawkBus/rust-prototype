use std::path::PathBuf;

use argh::FromArgs;

/// SquawkBus server.
#[derive(FromArgs)]
pub struct Options {
    /// ca file
    #[argh(option, short = 'c')]
    pub config: Option<PathBuf>,
}

impl Options {
    pub fn load() -> Self {
        argh::from_env()
    }
}
