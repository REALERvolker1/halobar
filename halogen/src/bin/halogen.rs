#![cfg_attr(not(feature = "bin"), no_std)]
#[cfg(not(feature = "bin"))]
fn main() {}

#[cfg(feature = "bin")]
pub mod halogen_stuff;

#[cfg(feature = "bin")]
fn main() -> halogen_stuff::R<()> {
    use std::io::IsTerminal;

    color_eyre::install()?;

    if let Some(log_level) = halogen_stuff::cli::CLI.log_level.tracing() {
        let logger = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(log_level)
            .pretty()
            .with_ansi(std::io::stdout().is_terminal())
            .with_target(true)
            .with_thread_ids(true)
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
            .with_writer(|| std::io::stdout());
        logger.init();
    }

    let future = async move {
        let (mut interface, stub) = halogen::interface::Interface::new().await?;

        let server_handle = tokio::spawn(async move {});

        halogen_stuff::async_main().await
    };

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(future)?;

    Ok(())
}
