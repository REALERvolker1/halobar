pub mod cli;

pub use color_eyre::eyre::{bail, Report};
pub use futures_util::{stream::FuturesUnordered, StreamExt};
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
            [$( (::tokio::signal::unix::signal(::tokio::signal::unix::SignalKind::$sigtype())?, stringify!($sigtyp)) ),+]
            // [$(async move { let $sigtype = ::tokio::signal::unix::signal(::tokio::signal::unix::SignalKind::$sigtype()); Ok::<_, Report>(())}),+]
            // tokio::select! {
                // Some(_) = s => {
                    // warn!(concat!("Received signal: ", stringify!($sigtype)))
                // }
            // }
        };
    }
    let mut signals = signals![interrupt, quit, terminate]
        .map(|(mut sig, name)| {
            let int = interface.clone();
            async move {
                let _ = sig.recv().await;
                // safety: We are a server
                unsafe { int.drop_path() };
                // TODO: Handle these properly
                panic!("Received signal: {name}");
            }
        })
        .into_iter()
        .collect::<FuturesUnordered<_>>();

    signals.next().await;

    bail!("Signal handler returned!")
}
