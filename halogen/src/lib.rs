/// API version 1
pub mod v1;

/// Use the current version's definitions
pub use v1::*;
/// The byte that is prepended to all messages, denoting the API version
pub const LATEST_API_VERSION: u8 = 1;

/// An internal library for stuff imported from other crates
mod imports;

pub mod interface;

use std::{env, path::PathBuf};

/// Try to get a valid socket path location.
///
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
    /// An error returned by [`get_socket_path`] when an environment variable is invalid. Returns the environment variable key.
    InvalidEnviron(&'static str),
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
    /// The current interface state is invalid -- maybe a client trying to be a server??
    InvalidState(interface::InterfaceState),
    /// Received a message from an unknown API version
    InvalidApiVersion(u8),
    /// Other errors that don't really fit well
    Internal(&'static str),
}
impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::InvalidSocketPath(p) => write!(f, "Invalid socket path: {}", p.display()),
            Self::InvalidEnviron(e) => write!(f, "Missing or invalid environment variable: {e}"),
            Self::Json(e) => e.fmt(f),
            Self::EarlyReturn => "Future returned too early".fmt(f),
            Self::SendError => "Error sending message through channel".fmt(f),
            Self::RecvError => "Error receiving message from channel".fmt(f),
            Self::JoinError(e) => e.fmt(f),
            Self::InvalidState(s) => write!(f, "Invalid interface state: {s:?}"),
            Self::InvalidApiVersion(v) => write!(f, "Invalid API version: {v}"),
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
