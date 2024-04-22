use super::*;
use clap::{Parser, ValueEnum};
use once_cell::sync::Lazy;
use tracing::Level;

pub static CLI: Lazy<Cli> = Lazy::new(|| Cli::parse());

#[derive(Debug, Parser)]
#[command(version, author, about, long_about = None)]
pub struct Cli {
    /// Set the logging level and verbosity
    #[arg(short, long, verbatim_doc_comment)]
    pub log_level: LogLevel,
    /// Manually override the socket path for debugging purposes(not recommended)
    #[arg(long, verbatim_doc_comment)]
    pub socket_path: Option<PathBuf>,
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
