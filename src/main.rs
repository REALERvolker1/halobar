pub mod logging;
pub mod prelude;

#[cfg(target_os = "linux")]
fn main() -> prelude::R<()> {
    color_eyre::install()?;
    let uname = nix::sys::utsname::uname()?;
    prelude::debug!("Running kernel {}", uname.release().to_string_lossy());

    Ok(())
}
