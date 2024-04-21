pub mod logging;
pub mod prelude;

#[cfg(target_os = "linux")]
fn main() -> prelude::R<()> {
    color_eyre::install()?;
    logging::start()?;
    let uname = nix::sys::utsname::uname()?;
    prelude::debug!("Running kernel {}", uname.release().to_string_lossy());

    let f = async move { Ok::<_, prelude::Report>(()) };

    Ok(())
}
