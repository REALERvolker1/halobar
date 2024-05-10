#![allow(async_fn_in_trait)]
#![deny(ambiguous_glob_imports)]
#![deny(large_assignments)]
#![deny(unsafe_code)]

pub mod globals;
pub mod modules;
mod prelude;
pub mod runtime;
pub mod types;

fn main() -> prelude::R<()> {
    color_eyre::install()?;

    let mut log_config = halogen::complete::LogConfig::default();
    log_config.level = halogen::complete::LogLevel::Debug;

    const NICE_TARGETS: [&str; 5] = ["iced", "wgpu", "zbus", "zbus_xml", "cosmic_text"];
    halogen::complete::init_log(&mut log_config, NICE_TARGETS);

    let uname = nix::sys::utsname::uname()?;
    prelude::info!("Running kernel {}", uname.release().to_string_lossy());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let rt = prelude::Arc::new(rt);

    let config = runtime::ModulesKnown::default();

    // rt.clone().block_on(modules::run(rt, config))?;
    rt.clone().block_on(runtime::run(rt.clone(), config))?;

    Ok(())
}
