use cli::cli::CLI;

pub mod cli;

fn main() -> cli::R<()> {
    color_eyre::install()?;

    halogen::complete::init_log(&CLI.logconfig, []);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    if CLI.server {}

    rt.block_on(async move {
        let (mut interface, stub) = halogen::interface::Interface::new().await?;

        let server_handle = tokio::spawn(async move {});

        cli::async_main().await
    })?;

    Ok(())
}
