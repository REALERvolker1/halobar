pub mod cli;

pub use color_eyre::eyre::{bail, Report};
pub use futures_util::{stream::FuturesUnordered, StreamExt};
use std::process::exit;
pub use std::{
    env,
    path::{Path, PathBuf},
};
pub use tracing::{debug, error, info, instrument, trace, warn};

pub type R<T> = color_eyre::Result<T>;

pub async fn async_main() -> R<()> {
    Ok(())
}

/// ONLY CALL THIS IF THE INTERFACE IS A SERVER!!!
async fn server_signal_handler(interface: halogen::interface::InterfaceStub) -> R<()> {
    macro_rules! signals {
        ($( $sigtype:tt ),+) => {
            [$( (::tokio::signal::unix::SignalKind::$sigtype(), stringify!($sigtyp)) ),+]
            // [$(async move { let $sigtype = ::tokio::signal::unix::signal(::tokio::signal::unix::SignalKind::$sigtype()); Ok::<_, Report>(())}),+]
            // tokio::select! {
                // Some(_) = s => {
                    // warn!(concat!("Received signal: ", stringify!($sigtype)))
                // }
            // }::tokio::signal::unix::signal()?
        };
    }
    let mut signals = signals![interrupt, quit, terminate]
        .map(|(sig, name)| {
            // Safety: This is an error that I can catch in development
            let mut signal_recv = tokio::signal::unix::signal(sig).unwrap();
            let int = interface.clone_of_arc();

            async move {
                let _ = signal_recv.recv().await;
                // safety: We are a server
                if let Err(e) = int.try_drop_path().await {
                    error!("Error while trying to drop socket path: {e}");
                }

                warn!("Received signal: {name}, shutting down...");
                exit(sig.as_raw_value())
            }
        })
        .into_iter()
        .collect::<FuturesUnordered<_>>();

    signals.next().await;

    bail!("Signal handler returned!")
}
