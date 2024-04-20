use crate::prelude::*;
use color_eyre::owo_colors::OwoColorize;
use std::fs;
use std::io::IsTerminal;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum_macros::EnumString,
    strum_macros::Display,
    strum_macros::VariantArray,
    strum_macros::EnumMessage,
)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum ColorOption {
    #[strum(message = "Always color terminal output")]
    Always,
    #[default]
    #[strum(message = "Only color output if running in a terminal")]
    Auto,
    #[strum(message = "Never color terminal output")]
    Never,
}
impl ColorOption {
    pub fn should_color(&self) -> bool {
        match self {
            Self::Always => true,
            Self::Auto => io::stdout().is_terminal(),
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
    strum_macros::VariantArray,
    strum_macros::EnumMessage,
)]
#[strum(serialize_all = "kebab-case")]
pub enum LogLevel {
    #[strum(message = "Disable all logging")]
    Disable,
    #[strum(message = "Only log errors")]
    Error,
    #[strum(message = "Log warnings as well as errors")]
    Warn,
    #[strum(message = "Log other information that might be useful")]
    #[cfg_attr(not(debug_assertions), default)]
    // #[default]
    Info,
    #[strum(message = "Enable debug logging. This can be very verbose!")]
    #[cfg_attr(debug_assertions, default)]
    Debug,
    #[strum(message = "Enable debug logging as well as trace logging. Even more verbose!")]
    Trace,
}

impl LogLevel {
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
            Self::Debug => LevelFilter::TRACE,
            Self::Trace => LevelFilter::TRACE,
            _ => LevelFilter::WARN,
        }
    }
}

#[derive(Debug, SmartDefault)]
pub struct LogConfig {
    pub level: LogLevel,
    pub should_color: bool,
    pub max_queued_messages: usize,
    pub logfile: Option<PathBuf>,
    pub guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}
impl LogConfig {}

static LOG_CONFIG: OnceCell<LogConfig> = OnceCell::new();

pub fn start() -> R<()> {
    let mut log_config = LogConfig::default();
    init_log(&mut log_config);

    if LOG_CONFIG.set(log_config).is_err() {
        bail!("Could not set global logging handle!")
    }

    Ok(())
}

fn init_log(config: &mut LogConfig) {
    let Some(tracing_level) = config.level.tracing() else {
        return;
    };

    #[cfg(debug_assertions)]
    const SPAN_EVENTS: FmtSpan = FmtSpan::ACTIVE;
    // let span_events: FmtSpan = FmtSpan::CLOSE | FmtSpan::ENTER;
    #[cfg(not(debug_assertions))]
    const SPAN_EVENTS: FmtSpan = FmtSpan::ENTER;

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing_level)
        .with_file(true)
        .with_level(true)
        .with_line_number(true)
        .with_span_events(SPAN_EVENTS)
        .with_target(true)
        .with_thread_ids(true)
        .with_ansi(config.should_color)
        .with_writer(io::stdout)
        .finish();

    let nice_filter = config.level.nice_level_filter();

    let subscriber = tracing_subscriber::filter::Targets::new()
        .with_default(config.level.level_filter())
        .with_target("cosmic_text", nice_filter)
        .with_target("iced", nice_filter)
        .with_target("wgpu", nice_filter)
        .with_target("zbus", nice_filter)
        .with_target("zbus_xml", nice_filter)
        .with_subscriber(subscriber);

    let subscriber =
        tracing_error::ErrorLayer::new(tracing_subscriber::fmt::format::Pretty::default())
            .with_subscriber(subscriber);

    match config.logfile {
        Some(ref logfile) => match fs::File::create(logfile) {
            Ok(f) => {
                let (appender, guard) =
                    tracing_appender::non_blocking::NonBlockingBuilder::default()
                        .buffered_lines_limit(config.max_queued_messages)
                        .lossy(true)
                        .finish(f);

                config.guard.replace(guard);

                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .with_level(true)
                    .with_span_events(SPAN_EVENTS)
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_writer(appender)
                    .with_thread_ids(true)
                    .with_subscriber(subscriber)
                    .init()
            }
            Err(e) => {
                println!("Error creating logfile: {e}");
                subscriber.init()
            }
        },
        None => subscriber.init(),
    }

    info!("Logging initialized");

    return;
}
