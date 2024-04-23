pub use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub use crate::{Error, Message};

pub use std::{
    convert::Infallible,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

pub use ahash::{HashMap, HashMapExt};
pub use futures_util::StreamExt;
pub use tokio::{
    io::{self, AsyncBufRead, AsyncRead, AsyncWrite},
    net::{UnixListener, UnixStream},
    sync::{mpsc, watch, Mutex, RwLock},
};
pub use tracing::{debug, error, instrument, trace, warn};

pub use smallvec::SmallVec;
// #[inline]
// pub fn to_string_pretty<S: ?Sized + Serialize>(value: &S) -> Result<String, serde_json::Error> {
//     serde_json::to_string_pretty(value)
// }

#[cfg(feature = "serde_json")]
pub use serde_json as json;
#[cfg(feature = "simd-json")]
pub use simd_json as json;

#[cfg(feature = "serde_json")]
#[inline]
pub fn from_string<D: DeserializeOwned>(input_string: &str) -> Result<D, Error> {
    let out = serde_json::from_str(input_string)?;
    Ok(out)
}

#[cfg(feature = "simd-json")]
#[inline]
pub fn from_bytes<D: DeserializeOwned>(input: &mut [u8]) -> Result<D, Error> {
    let out = unsafe { simd_json::from_slice(input) }?;
    Ok(out)
}
