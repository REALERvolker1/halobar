/// API version 1
pub mod v1;
/// Use the current version's definitions
pub use v1::*;

/// An internal library for stuff imported from other crates
pub(crate) mod imports;

pub mod server;

use std::{env, path::PathBuf};

/// Override the path to the socket.
pub const SOCKET_OVERRIDE_VARIABLE: &str = "HALOGEN_SOCK";

/// Try to get a valid socket path location
pub fn get_socket_path() -> Result<PathBuf, Error> {
    if let Some(env_path) = env::var_os(SOCKET_OVERRIDE_VARIABLE) {
        let env_path = PathBuf::from(env_path);
        return if env_path.exists() {
            Ok(env_path)
        } else {
            Err(Error::InvalidSocketPath(env_path))
        };
    }

    let mut runtime_dir = match env::var_os("XDG_RUNTIME_DIR") {
        Some(p) => PathBuf::from(p),
        None => return Err(Error::InvalidSocketPath(PathBuf::new())),
    };

    let path_metadata = runtime_dir.metadata()?;
    if !path_metadata.is_dir() || path_metadata.permissions().readonly() {
        return Err(Error::InvalidSocketPath(runtime_dir));
    }

    runtime_dir.push("halogen.sock");

    Ok(runtime_dir)
}

#[cfg(feature = "serde_json")]
use serde_json::Error as JsonError;
#[cfg(feature = "simd-json")]
use simd_json::Error as JsonError;

/// All the errors returned by this crate
#[derive(Debug)]
pub enum Error {
    /// An error wrapping [`std::io::Error`]
    Io(std::io::Error),
    /// An error returned by [`get_socket_path`] when the socket path is invalid.
    InvalidSocketPath(PathBuf),
    /// An error that occured when parsing json
    Json(JsonError),
}
impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::InvalidSocketPath(p) => write!(f, "Invalid socket path: {}", p.display()),
            Self::Json(e) => e.fmt(f),
        }
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<JsonError> for Error {
    fn from(value: JsonError) -> Self {
        Self::Json(value)
    }
}
