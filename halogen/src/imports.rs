use serde::de::DeserializeOwned;

pub(crate) use crate::{Error, Message, Variant};

pub(crate) use std::{convert::Infallible, path::PathBuf, str::FromStr, sync::Arc};

pub(crate) use ahash::{HashMap, HashMapExt};
pub(crate) use serde::{Deserialize, Serialize};

#[cfg(feature = "monoio")]
mod monoio_types {
    pub use flume::unbounded;
    pub type ServerSender<T> = flume::Sender<T>;
    pub type ServerReceiver<T> = flume::Receiver<T>;

    pub use monoio::{
        buf::IoBufMut,
        io::{
            self,
            stream::{Stream, StreamExt},
            AsyncBufRead, AsyncBufReadExt, AsyncReadRent, AsyncReadRentExt,
        },
        join,
        net::UnixStream,
        select, try_join,
    };
}
#[cfg(feature = "monoio")]
pub use monoio_types::*;

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
    let out = unsafe { simd_json::from_str(owned_string.as_mut_str()) }?;
    Ok(out)
}
