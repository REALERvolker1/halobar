pub use serde::{Deserialize, Serialize};

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

#[cfg(all(not(target_arch = "x86_64"), feature = "simd-json"))]
compile_error!("simd-json feature is only available for x86_64 architectures!");
