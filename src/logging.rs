use crate::prelude::*;
use color_eyre::owo_colors::OwoColorize;
use std::io::IsTerminal;
use strum::{EnumMessage, VariantArray};
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
            Self::Debug => LevelFilter::DEBUG,
            Self::Trace => LevelFilter::TRACE,
            _ => LevelFilter::WARN,
        }
    }
}

#[derive(Debug)]
pub struct LogConfig {
    pub level: LogLevel,
    pub should_color: bool,
    pub max_queued_messages: usize,
    pub logfile: Option<PathBuf>,
    pub guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}
impl LogConfig {
    /// Parses CLI args, returns a new instance
    pub fn new() -> R<Self> {
        let mut level = None;
        let mut should_color = ColorOption::default();
        let mut max_queued_messages = None;
        let mut logfile = None;

        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--logfile" => match args.next() {
                    Some(f) => {
                        let fpath = Path::new(&f);
                        if fpath.try_exists()? {
                            bail!("Error, logfile {f} already exists!");
                        }

                        let Some(parent) = fpath.parent() else {
                            bail!("Error, logfile {f} has no parent directory!");
                        };

                        if !parent.try_exists()? {
                            fs::create_dir_all(parent).map_err(|e| {
                                eyre!("Could not create parent directory of logfile {f}: {e}")
                            })?;
                        }

                        logfile.replace(PathBuf::from(f));
                    }
                    None => bail!("Error, no logfile provided!"),
                },
                "--no-logfile" => {
                    logfile.take();
                }
                "--max-queued-messages" => match args.next() {
                    Some(a) => {
                        let max = a
                            .parse::<usize>()
                            .map_err(|e| eyre!("Error parsing max queued messages: {e}"))?;
                        max_queued_messages.replace(max);
                    }
                    None => bail!("Error, no integer value provided for the max queued messages!"),
                },
                "--log-level" => match args.next() {
                    Some(l) => {
                        let chosen_level = LogLevel::from_str(&l)
                            .map_err(|_| eyre!("Error, invalid log level: {l}"))?;
                        level.replace(chosen_level);
                    }
                    None => bail!("Error, no log level provided!"),
                },
                "--color" => match args.next() {
                    Some(a) => {
                        let color = ColorOption::from_str(&a)
                            .map_err(|_| eyre!("Error, invalid color option: {a}"))?;
                        should_color = color;
                    }
                    None => {
                        // does not require another arg
                        should_color = ColorOption::Always
                    }
                },
                _ => bail!("Error: Invalid arg passed: {arg}"),
            }
        }

        Ok(Self {
            level: level.unwrap_or_default(),
            should_color: should_color.should_color(),
            max_queued_messages: max_queued_messages.unwrap_or(64),
            logfile,
            guard: None,
        })
    }
    pub fn help() -> String {
        let log_levels = LogLevel::VARIANTS
            .into_iter()
            .map(|v| format!("\t{}: {}", v.underline(), v.get_message().unwrap()))
            .collect::<Vec<_>>()
            .join("\n");

        let color_options = ColorOption::VARIANTS
            .into_iter()
            .map(|v| format!("\t{}: {}", v.underline(), v.get_message().unwrap()))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "\x1b[0mUsage: {} [--options]

Available options:

{} \t Set the logfile path
{} \t Disable logging to file (default)
{} \t Set the maximum number of messages to log to the logfile before dropping

{} \t Set the log level
Available levels:
{log_levels}

{} \t Choose when to color the terminal
Available choices:
{color_options}",
            env!("CARGO_BIN_NAME"),
            "--logfile".bold(),
            "--no-logfile".bold(),
            "--max-queued-messages".bold(),
            "--log-level".bold(),
            "--color".bold()
        )
    }
}

static LOG_CONFIG: OnceCell<LogConfig> = OnceCell::new();

pub fn start() -> R<()> {
    let mut log_config = match LogConfig::new() {
        Ok(c) => c,
        Err(e) => {
            bail!("{}\n\n{e}", LogConfig::help());
            // std::process::exit(1);
        }
    };
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

    // let nice_filter = log_level.nice_level_filter();

    // let iced_disablement = tracing_subscriber::filter::Targets::new()
    //     .with_default(log_level.level_filter())
    //     .with_target("cosmic_text", nice_filter)
    //     .with_target("iced", nice_filter)
    //     .with_target("wgpu", nice_filter)
    //     .with_target("zbus", nice_filter)
    //     .with_target("zbus_xml", nice_filter)
    //     .with_subscriber(term_subscriber);

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
