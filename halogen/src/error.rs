use crate::imports::*;

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
    Json(json::Error),
    /// An error that is returned from futures that ended too early
    EarlyReturn,
    /// An error sending data through a channel
    SendError,
    /// An error receiving data from a channel
    RecvError,
    /// An error joining a task
    JoinError(tokio::task::JoinError),
    /// The current interface state is invalid -- maybe a client trying to be a server??
    InvalidState(crate::interface::InterfaceState),
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
impl From<json::Error> for Error {
    fn from(value: json::Error) -> Self {
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
