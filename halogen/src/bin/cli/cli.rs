use super::*;
use clap::{Parser, Subcommand};
use once_cell::sync::Lazy;

// pub static CLI: Lazy<Cli> = Lazy::new(|| Cli::new());

#[derive(Debug, Parser)]
#[command(version, author, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub logconfig: halogen::complete::LogConfig,
    /// Manually override the socket path for debugging purposes (not recommended)
    #[arg(long, verbatim_doc_comment)]
    pub socket_path: Option<PathBuf>,
    #[arg(
        long,
        help = "Start halogen in SERVER mode",
        long_help = "Start halogen in SERVER mode. Please note that there can only be one server per socket!"
    )]
    pub server: bool,
}
impl Cli {
    /// Create a new [`Cli`], parsing args and doing other misc tasks
    #[inline]
    pub fn new() -> Self {
        let me = Self::parse();
        me
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Server,
    Msg {},
}
