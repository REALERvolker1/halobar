#![cfg_attr(not(feature = "bin"), no_std)]
#[cfg(not(feature = "bin"))]
fn main() {}

#[cfg(feature = "bin")]
pub mod halogen_stuff;

#[cfg(feature = "bin")]
fn main() -> halogen_stuff::R<()> {
    use std::io::IsTerminal;

    use tracing_subscriber::fmt::format::FmtSpan;

    let logger = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .pretty()
        .with_ansi(std::io::stdout().is_terminal())
        .with_target(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::ACTIVE)
        .with_writer(|| std::io::stdout());
    logger.init();

    let future = async move {
        let (mut server, stub) = halogen::interface::Interface::new().await?;

        let server_handle = tokio::spawn(async move { server.server().await });

        // safety: trust me bro
        server_handle.await.unwrap()?;
        Ok::<_, halogen::Error>(())
    };

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(future)?;

    Ok(())
}
