use super::*;
use clap::{Parser, Subcommand, ValueEnum};
use once_cell::sync::Lazy;
use tracing::Level;

pub static CLI: Lazy<Cli> = Lazy::new(|| Cli::parse());

#[derive(Debug, Parser)]
#[command(version, author, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, help = "Set the logging level and verbosity")]
    pub log_level: LogLevel,
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

#[derive(Debug, Default, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Quiet,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}
impl LogLevel {
    /// Get this as a Tracing level
    #[inline]
    pub const fn tracing(&self) -> Option<Level> {
        let level = match self {
            Self::Quiet => return None,
            Self::Error => Level::ERROR,
            Self::Warn => Level::WARN,
            Self::Info => Level::INFO,
            Self::Debug => Level::DEBUG,
            Self::Trace => Level::TRACE,
        };

        Some(level)
    }
}
