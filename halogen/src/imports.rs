use serde::de::DeserializeOwned;

pub(crate) use crate::{Error, Message, Variant};

pub(crate) use std::{path::PathBuf, sync::Arc};

#[cfg(feature = "flume")]
pub(crate) use flume::unbounded;

#[cfg(feature = "flume")]
pub type ServerSender<T> = flume::Sender<T>;
#[cfg(feature = "flume")]
pub type ServerReceiver<T> = flume::Receiver<T>;

// #[inline]
// pub fn to_string_pretty<S: ?Sized + Serialize>(value: &S) -> Result<String, serde_json::Error> {
//     serde_json::to_string_pretty(value)
// }

#[cfg(all(test, feature = "serde_json"))]
pub use serde_json::to_string_pretty;

#[cfg(all(test, feature = "simd-json"))]
pub use simd_json::to_string_pretty;

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
    unsafe { simd_json::from_str(owned_string.as_mut_str()) }
}
