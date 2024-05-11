use clap::{Args, ValueEnum};
use once_cell::sync::OnceCell;
use tracing::level_filters::LevelFilter;

pub(crate) use crate::imports::*;
use std::fs;
use std::io::IsTerminal;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

/// The configuration for logging
#[derive(Debug, Default, Args)]
pub struct LogConfig {
    /// Set the verbosity of logging
    #[arg(verbatim_doc_comment, short, long, default_value_t = LogLevel::default())]
    pub level: LogLevel,
    /// Choose when to print ANSI terminal colors with log output
    #[arg(verbatim_doc_comment, short, long, default_value_t = ColorOption::default())]
    pub color: ColorOption,
    /// Log to file as well as output
    #[arg(verbatim_doc_comment, long)]
    pub logfile: Option<PathBuf>,
}

/// The console output of my choice
#[inline(always)]
pub fn console() -> std::io::Stdout {
    std::io::stdout()
}

static WORKER_GUARD: OnceCell<tracing_appender::non_blocking::WorkerGuard> = OnceCell::new();

/// The maximum number of queued messages to keep before dropping
pub const MAX_QUEUED_MESSAGES: usize = 64;

#[cfg(debug_assertions)]
const FMT_SPAN_EVENTS: FmtSpan = FmtSpan::ACTIVE;
#[cfg(not(debug_assertions))]
const FMT_SPAN_EVENTS: FmtSpan = FmtSpan::ENTER;

/// Initialize logging with tracing. This calls the global init function!
///
/// nice_targets is meant to be a const array.
#[inline]
pub fn init_log<const N: usize>(config: &LogConfig, nice_targets: [&str; N]) {
    let Some(tracing_level) = config.level.tracing() else {
        return;
    };
    let should_color = config.color.should_color();

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing_level)
        .with_file(true)
        .with_level(true)
        .with_line_number(true)
        .with_span_events(FMT_SPAN_EVENTS)
        .with_target(true)
        .with_thread_ids(true)
        .with_ansi(should_color)
        .with_writer(console)
        .finish();

    let nice_filter = config.level.nice_level_filter();

    let subscriber = tracing_subscriber::filter::Targets::new()
        .with_default(config.level.level_filter())
        .with_targets(nice_targets.map(|t| (t, nice_filter)))
        .with_subscriber(subscriber);

    let subscriber =
        tracing_error::ErrorLayer::new(tracing_subscriber::fmt::format::Pretty::default())
            .with_subscriber(subscriber);

    match config.logfile {
        Some(ref logfile) => match fs::File::create(logfile) {
            Ok(f) => {
                let (appender, guard) =
                    tracing_appender::non_blocking::NonBlockingBuilder::default()
                        .buffered_lines_limit(MAX_QUEUED_MESSAGES)
                        .lossy(true)
                        .finish(f);

                WORKER_GUARD
                    .set(guard)
                    .expect("Could not set the global tracing_appender worker guard!");

                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .with_level(true)
                    .with_span_events(FMT_SPAN_EVENTS)
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_writer(appender)
                    .with_thread_ids(true)
                    .with_subscriber(subscriber)
                    .init()
            }
            Err(e) => {
                subscriber.init();
                error!("Error creating logfile: {e}");
            }
        },
        None => subscriber.init(),
    }

    tracing::info!("Logging initialized");
    tracing::trace!("Receiving trace messages!");

    return;
}

/// Choose when to color the terminal output
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum_macros::EnumString,
    strum_macros::Display,
    Serialize,
    Deserialize,
    ValueEnum,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ColorOption {
    /// Always color terminal output
    #[clap(verbatim_doc_comment)]
    Always,
    #[default]
    /// Only color output if running in a terminal
    #[clap(verbatim_doc_comment)]
    Auto,
    /// Never color terminal output
    #[clap(verbatim_doc_comment)]
    Never,
}
impl ColorOption {
    /// Determine if the logging output should show ansi colors
    pub fn should_color(&self) -> bool {
        match self {
            Self::Always => true,
            Self::Auto => console().is_terminal(),
            Self::Never => false,
        }
    }
}

/// The logging level. Determines what kind of log messages will be printed.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum_macros::Display,
    strum_macros::EnumString,
    Serialize,
    Deserialize,
    ValueEnum,
)]
#[strum(serialize_all = "kebab-case")]
pub enum LogLevel {
    /// Disable all logging
    #[clap(verbatim_doc_comment)]
    Disable,
    /// Only log errors
    #[clap(verbatim_doc_comment)]
    Error,
    /// Log warnings as well as errors
    #[clap(verbatim_doc_comment)]
    Warn,
    /// Log other information that might be useful
    #[cfg_attr(not(debug_assertions), default)]
    // #[default]
    #[clap(verbatim_doc_comment)]
    Info,
    /// Enable debug logging. This can be very verbose!
    #[cfg_attr(debug_assertions, default)]
    #[clap(verbatim_doc_comment)]
    Debug,
    /// Enable debug logging as well as trace logging. Even more verbose!
    #[clap(verbatim_doc_comment)]
    Trace,
}
impl LogLevel {
    /// Get the tracing level corresponding to this
    pub const fn tracing(&self) -> Option<tracing::Level> {
        let lvl = match self {
            Self::Disable => return None,
            Self::Error => tracing::Level::ERROR,
            Self::Warn => tracing::Level::WARN,
            Self::Info => tracing::Level::INFO,
            Self::Debug => tracing::Level::DEBUG,
            Self::Trace => tracing::Level::TRACE,
        };
        return Some(lvl);
    }
    /// Get the level filter corresponding to this
    pub const fn level_filter(&self) -> LevelFilter {
        match self {
            Self::Disable => LevelFilter::OFF,
            Self::Error => LevelFilter::ERROR,
            Self::Warn => LevelFilter::WARN,
            Self::Info => LevelFilter::INFO,
            Self::Debug => LevelFilter::DEBUG,
            Self::Trace => LevelFilter::TRACE,
        }
    }
    /// The filter to use for other crates that spew garbage, so they are nicer
    ///
    /// This is here so I can use level filter but keep these a bit less verbose.
    pub const fn nice_level_filter(&self) -> LevelFilter {
        match self {
            Self::Disable => LevelFilter::OFF,
            Self::Error => LevelFilter::ERROR,
            Self::Debug => LevelFilter::WARN,
            Self::Trace => LevelFilter::TRACE,
            _ => LevelFilter::WARN,
        }
    }
}
