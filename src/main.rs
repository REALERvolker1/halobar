pub mod prelude;

#[cfg(target_os = "linux")]
fn main() -> prelude::R<()> {
    color_eyre::install()?;

    let mut log_config = halogen::halobar_integration::LogConfig::default();
    halogen::halobar_integration::init_log(&mut log_config);

    let uname = nix::sys::utsname::uname()?;
    prelude::debug!("Running kernel {}", uname.release().to_string_lossy());

    let f = async move { Ok::<_, prelude::Report>(()) };

    Ok(())
}
