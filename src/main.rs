pub mod prelude;

#[cfg(target_os = "linux")]
fn main() -> prelude::R<()> {
    color_eyre::install()?;

    let mut log_config = halogen::halobar_integration::LogConfig::default();

    const NICE_TARGETS: [&str; 5] = ["iced", "wgpu", "zbus", "zbus_xml", "cosmic_text"];
    halogen::halobar_integration::init_log(&mut log_config, NICE_TARGETS);

    let uname = nix::sys::utsname::uname()?;
    prelude::debug!("Running kernel {}", uname.release().to_string_lossy());

    let f = async move { Ok::<_, prelude::Report>(()) };

    Ok(())
}
