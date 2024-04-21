/// API version 1
pub mod v1;
/// Use the current version's definitions
pub use v1::*;

/// An internal library for stuff imported from other crates
mod imports;

pub mod interface;

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
    /// An error that is returned from futures that ended too early
    EarlyReturn,
    /// An error sending data through a channel
    SendError,
    /// An error receiving data from a channel
    RecvError,
    /// An error joining a task
    JoinError(tokio::task::JoinError),
    InvalidState(interface::InterfaceState),
    /// Other errors that don't really fit well
    Internal(&'static str),
}
impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::InvalidSocketPath(p) => write!(f, "Invalid socket path: {}", p.display()),
            Self::Json(e) => e.fmt(f),
            Self::EarlyReturn => "Future returned too early".fmt(f),
            Self::SendError => "Error sending message through channel".fmt(f),
            Self::RecvError => "Error receiving message from channel".fmt(f),
            Self::JoinError(e) => e.fmt(f),
            Self::InvalidState(s) => write!(f, "Invalid interface state: {s:?}"),
            Self::Internal(e) => e.fmt(f),
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
impl From<tokio::task::JoinError> for Error {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::JoinError(value)
    }
}

macro_rules! senderr {
    ($($module:tt),+) => {
        $(
            impl<T> From<::tokio::sync::$module::error::SendError<T>> for Error {
                fn from(_: ::tokio::sync::$module::error::SendError<T>) -> Self {
                    Self::SendError
                }
            }
        )+
    };
    (flume $($err:tt),+) => {
        $(
            impl<T> From<::flume::$err<T>> for Error {
                fn from(_: ::flume::$err<T>) -> Self {
                    Self::SendError
                }
            }
        )+
    };
    (flume recv $($err:tt),+) => {
        $(
            impl From<::flume::$err> for Error {
                fn from(_: ::flume::$err) -> Self {
                    Self::RecvError
                }
            }
        )+
    }
}
senderr![mpsc, broadcast, watch];
senderr![flume SendError, TrySendError, SendTimeoutError];
senderr![flume recv RecvError, RecvTimeoutError, TryRecvError];

// /// A set of futures that poll but also let you insert
// #[derive(Debug)]
// pub struct BruhSet<Fut: std::future::Future<Output = T>, T> {
//     cursor: usize,
//     futures: Vec<Fut>,
//     insert_next: Option<Fut>,
// }
// impl<Fut: std::future::Future<Output = T>, T> Default for BruhSet<Fut, T> {
//     fn default() -> Self {
//         Self {
//             cursor: 0,
//             futures: Vec::new(),
//             insert_next: None,
//         }
//     }
// }
// impl<Fut: std::future::Future<Output = T>, T> BruhSet<Fut, T> {
//     /// Create a new empty instance
//     #[inline]
//     pub fn new() -> Self {
//         Self::default()
//     }
// }
// impl<Fut: std::future::Future<Output = T>, T> futures_util::Stream for BruhSet<Fut, T> {
//     type Item = T;
//     fn poll_next(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Option<Self::Item>> {
//         if self.futures.is_empty() {
//             return Poll::Ready(None);
//         }

//         if self.cursor > self.futures.len() {
//             self.cursor = self.futures.len();
//         }

//         let next = self.futures.get(self.cursor).unwrap();
//         self.cursor = self.cursor.wrapping_add(1);
//         next
//     }
// }
