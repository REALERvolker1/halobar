#![allow(async_fn_in_trait)]

pub mod modules;
pub mod prelude;

#[cfg(target_os = "linux")]
fn main() -> prelude::R<()> {
    color_eyre::install()?;

    let mut log_config = halogen::complete::LogConfig::default();
    log_config.level = halogen::complete::LogLevel::Trace;

    const NICE_TARGETS: [&str; 5] = ["iced", "wgpu", "zbus", "zbus_xml", "cosmic_text"];
    halogen::complete::init_log(&mut log_config, NICE_TARGETS);

    let uname = nix::sys::utsname::uname()?;
    prelude::debug!("Running kernel {}", uname.release().to_string_lossy());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(modules::run(&rt))?;

    Ok(())
}
