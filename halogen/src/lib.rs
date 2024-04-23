/// API version 1
pub mod v1;

/// Use the current version's definitions
pub use v1::*;
/// The byte that is prepended to all messages, denoting the API version
pub const LATEST_API_VERSION: u8 = 1;

#[cfg(feature = "halobar_integration")]
/// Both halobar and halogen-cli require tracing-subscriber and the contents of this subcrate
pub mod halobar_integration;

/// An internal library for stuff imported from other crates
#[cfg(feature = "bin")]
pub mod imports;

#[cfg(not(feature = "bin"))]
mod imports;

/// The main interface
pub mod interface;

mod error;
pub use error::Error;

use std::{env, path::PathBuf};

/// Try to get a valid socket path location.
///
/// First tries the environment variable `${HALOGEN_SOCK}`, then tries
/// `${XDG_RUNTIME_DIR}/halogen/halogen${XDG_SESSION_ID}.sock`
pub fn get_socket_path() -> Result<PathBuf, Error> {
    if let Some(env_path) = env::var_os("HALOGEN_SOCK") {
        let env_path = PathBuf::from(env_path);
        tracing::debug!(
            "Loading path from environment variable HALOGEN_SOCK: {}",
            env_path.display()
        );
        return Ok(env_path);
    }
    tracing::trace!("Environment variable HALOGEN_SOCK was undefined");

    let mut path = match env::var_os("XDG_RUNTIME_DIR") {
        Some(p) => PathBuf::from(p),
        None => return Err(Error::InvalidEnviron("XDG_RUNTIME_DIR")),
    };

    let path_metadata = path.metadata()?;
    if !path_metadata.is_dir() || path_metadata.permissions().readonly() {
        return Err(Error::InvalidSocketPath(path));
    }
    path.push("halogen");

    let mut socket_name = "halosock".to_owned();

    let session_id =
        env::var("XDG_SESSION_ID").map_err(|_| Error::InvalidEnviron("XDG_SESSION_ID"))?;

    // make sure the path exists
    if !path.is_dir() {
        std::fs::create_dir_all(&path)?;
    }

    socket_name.push_str(&session_id);
    socket_name.push_str(".sock");

    path.push(socket_name);

    Ok(path)
}
