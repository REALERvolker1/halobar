use super::*;
use clap::{Parser, Subcommand};
use once_cell::sync::Lazy;

pub static CLI: Lazy<Cli> = Lazy::new(|| Cli::parse());

#[derive(Debug, Parser)]
#[command(version, author, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub logconfig: halogen::halobar_integration::LogConfig,
    #[arg(
        long,
        help = "Manually override the socket path for debugging purposes (not recommended)"
    )]
    pub socket_path: Option<PathBuf>,
    #[arg(
        long,
        help = "Start halogen in SERVER mode",
        long_help = "Start halogen in SERVER mode. Please note that there can only be one server per socket!"
    )]
    pub server: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Server,
    Msg {},
}
