pub mod cli;

pub use color_eyre::eyre::{bail, Report};
pub use halogen::imports::*;
pub use tracing::info;

pub type R<T> = color_eyre::Result<T>;

pub async fn async_main() -> R<()> {
    Ok(())
}

/// ONLY CALL THIS IF THE INTERFACE IS A SERVER!!!
pub async fn server_signal_handler(interface: halogen::interface::InterfaceStub) -> ! {
    macro_rules! signals {
        ($( $sigtype:tt ),+) => {
            [$( (::tokio::signal::unix::SignalKind::$sigtype(), stringify!($sigtyp)) ),+]
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
                if let Err(e) = int.drop_path() {
                    error!("Error while trying to drop socket path: {e}");
                }

                warn!("Received signal: {name}, shutting down...");
                std::process::exit(sig.as_raw_value())
            }
        })
        .into_iter()
        .collect::<FuturesUnordered<_>>();

    loop {
        signals.next().await;
    }
}
