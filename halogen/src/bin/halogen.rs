pub mod cli;

use std::io::IsTerminal;

fn main() -> cli::R<()> {
    color_eyre::install()?;

    halogen::halobar_integration::init_log(&cli::cli::CLI.logconfig, []);

    let future = async move {
        let (mut interface, stub) = halogen::interface::Interface::new().await?;

        let server_handle = tokio::spawn(async move {});

        cli::async_main().await
    };

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(future)?;

    Ok(())
}
