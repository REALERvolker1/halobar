#![cfg_attr(not(feature = "bin"), no_std)]
#[cfg(not(feature = "bin"))]
fn main() {}

#[cfg(feature = "bin")]
fn main() -> Result<(), halogen::Error> {
    use std::io::IsTerminal;

    let logger = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .pretty()
        .with_ansi(std::io::stdout().is_terminal())
        .with_writer(|| std::io::stdout());
    logger.init();

    let future = async move {
        let mut server = halogen::interface::Interface::new().await?;

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
