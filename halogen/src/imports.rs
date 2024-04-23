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
pub use tokio::net::{UnixListener, UnixStream};
pub use tracing::{debug, error, instrument, trace, warn};

pub use smallvec::SmallVec;

#[cfg(feature = "serde_json")]
pub use serde_json as json;
#[cfg(feature = "simd-json")]
pub use simd_json as json;

// #[inline]
// pub fn from_bytes<D: DeserializeOwned>(input: &mut [u8]) -> Result<D, Error> {
//     // serde_json does not mark this as unsafe, while simd-json does.
//     #[cfg_attr(feature = "serde_json", allow(unused_unsafe))]
//     let out = unsafe { json::from_slice(input) }?;
//     Ok(out)
// }
