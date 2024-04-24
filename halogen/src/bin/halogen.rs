pub mod cli;

fn main() -> cli::R<()> {
    color_eyre::install()?;

    let cli = cli::cli::Cli::new();

    halogen::complete::init_log(&cli.logconfig, []);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move { cli::async_main(cli).await })?;

    Ok(())
}
