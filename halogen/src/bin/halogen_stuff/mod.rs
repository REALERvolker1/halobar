use color_eyre::eyre::{bail, Report};
use tracing::warn;

pub type R<T> = color_eyre::Result<T>;

pub async fn async_main() -> R<()> {
    Ok(())
}

async fn signal_handler(interface: halogen::interface::InterfaceStub) -> R<()> {
    macro_rules! signals {
        ($( $sigtype:tt ),+) => {
            [$( (::tokio::signal::unix::signal(::tokio::signal::unix::SignalKind::$sigtype())?, stringify!($sigtyp)) ),+]
            // [$(async move { let $sigtype = ::tokio::signal::unix::signal(::tokio::signal::unix::SignalKind::$sigtype()); Ok::<_, Report>(())}),+]
            // tokio::select! {
                // Some(_) = s => {
                    // warn!(concat!("Received signal: ", stringify!($sigtype)))
                // }
            // }
        };
    }
    let signals = signals![interrupt, quit, terminate].map(|(mut sig, name)| async move {
        let _ = sig.recv().await;
        warn!("Received signal: {name}")
    });

    bail!("Signal handler returned!")
}
