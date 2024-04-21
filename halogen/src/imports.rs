use serde::de::DeserializeOwned;

pub(crate) use crate::{Error, Message, Variant};

pub(crate) use std::{convert::Infallible, path::PathBuf, str::FromStr, sync::Arc};

pub(crate) use ahash::{HashMap, HashMapExt};
pub(crate) use serde::{Deserialize, Serialize};

pub use tokio::{
    io::{self, AsyncBufRead, AsyncRead, AsyncWrite},
    join,
    net::{UnixListener, UnixStream},
    select,
    sync::{mpsc, watch},
    try_join,
};

// #[inline]
// pub fn to_string_pretty<S: ?Sized + Serialize>(value: &S) -> Result<String, serde_json::Error> {
//     serde_json::to_string_pretty(value)
// }

#[cfg(feature = "serde_json")]
pub use serde_json::to_string;

#[cfg(feature = "simd-json")]
pub use simd_json::to_string;

#[cfg(feature = "serde_json")]
#[inline]
pub fn from_string<D: DeserializeOwned>(input_string: &str) -> Result<D, Error> {
    let out = serde_json::from_str(input_string)?;
    Ok(out)
}

#[cfg(feature = "simd-json")]
#[inline]
pub fn from_string<D: DeserializeOwned>(input_string: &str) -> Result<D, Error> {
    let mut owned_string = input_string.to_owned();
    let out = unsafe { simd_json::from_str(owned_string.as_mut_str()) }?;
    Ok(out)
}
