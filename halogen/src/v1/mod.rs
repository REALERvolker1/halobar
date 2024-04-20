use std::{convert::Infallible, path::PathBuf, str::FromStr};

use ahash::{HashMap, HashMapExt};
use serde::{Deserialize, Serialize};

/// A recognized type that can be sent through IPC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Variant {
    /// A literal string of text. Only meant to be used for Strings.
    ///
    /// Please use [`Variant::Other`] for custom data types,
    /// and [`Variant::Path`] for paths to files or directories.
    String(String),
    /// A verified filepath
    Path(PathBuf),

    /// A boolean value, Accepts true/false, on/off, 1/0, case-insensitive
    Bool(bool),
    /// signed integer
    Iint(i64),
    /// Unsigned integer
    Uint(u64),
    /// Double-precision floating point number
    Float(f64),

    /// Multiple other Variants in a single-dimensional vector
    Vec(Vec<Box<Variant>>),
    /// Multiple other variants in a key-value map. The keys can only be Strings.
    Map(HashMap<String, Box<Variant>>),

    /// Another data type, possibly serialized as a String.
    Other(String),
}
impl FromStr for Variant {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::String(s.to_owned()))
    }
}
macro_rules! impl_inner {
    ($($ty:ty: $variant:tt),+) => {
        $(
            impl From<$ty> for Variant {
                fn from(value: $ty) -> Self {
                    Self::$variant(value)
                }
            }
        )+
    };
    (@iint $($ty:ty),+) => {
        $(
            impl From<$ty> for Variant {
                fn from(value: $ty) -> Self {
                    Self::Iint(value as i64)
                }
            }
        )+
    };
    (@uint $($ty:ty),+) => {
        $(
            impl From<$ty> for Variant {
                fn from(value: $ty) -> Self {
                    Self::Uint(value as u64)
                }
            }
        )+
    };
}
impl_inner![String: String, PathBuf: Path, bool: Bool, i64: Iint, u64: Uint, f64: Float];
impl_inner![@iint i8, i16, i32, isize];
impl_inner![@uint u8, u16, u32, usize];
impl From<Vec<Variant>> for Variant {
    fn from(value: Vec<Variant>) -> Self {
        Self::from_iterator(value)
    }
}
impl<S: Into<String>, V: Into<Variant>, R> From<std::collections::HashMap<S, V, R>> for Variant {
    fn from(value: std::collections::HashMap<S, V, R>) -> Self {
        Self::Map(
            value
                .into_iter()
                .map(|(k, v)| (k.into(), Box::new(v.into())))
                .collect(),
        )
    }
}

impl Variant {
    /// Create a [`Variant::Vec`] from a Vector or Iterator-like.
    #[cfg_attr(feature = "tracing", ::tracing::instrument(level = "trace", skip_all))]
    pub fn from_iterator<I: IntoIterator<Item = Self>>(vec: I) -> Self {
        Self::Vec(vec.into_iter().map(|v| Box::new(v)).collect())
    }
    /// Get a wholly owned Vec from this Variant, if it is a Vec type. Otherwise returns the provided Variant.
    #[cfg_attr(feature = "tracing", ::tracing::instrument(level = "trace", skip_all))]
    pub fn unboxed_vec(self) -> Result<Vec<Variant>, Variant> {
        if let Self::Vec(v) = self {
            let unboxed = v.into_iter().map(|b| *b).collect();
            return Ok(unboxed);
        }
        Err(self)
    }
    /// Get a wholly owned hashmap out of this Variant. Only works for Map types, please see [`Variant::unboxed_vec`]
    #[cfg_attr(feature = "tracing", ::tracing::instrument(level = "trace", skip_all))]
    pub fn unboxed_hashmap(self) -> Result<HashMap<String, Variant>, Variant> {
        if let Self::Map(m) = self {
            let unboxed = m.into_iter().map(|(k, v)| (k, *v)).collect();
            return Ok(unboxed);
        }
        Err(self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// The API version that created this Message
    pub version: u8,
    /// The current status. Signifies the current state of things.
    pub status: Status,
    /// The name of this message
    pub key: String,
    /// Inner data payload
    pub data: HashMap<String, Variant>,
}
impl Message {
    // pub fn new(key: &str)
    /// Deserialize a message from raw json
    #[cfg_attr(feature = "tracing", ::tracing::instrument(level = "debug", skip_all))]
    pub fn try_from_raw(json: &str) -> Result<Self, crate::imports::Error> {
        crate::imports::from_string(json)
    }
}

/// A general-purpose status
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Status {
    Good,
    #[default]
    Normal,
    Warn,
    Bad,
    Critical,
}
impl Status {
    /// Log a tracing message with the tracing macros, using this [`Status`] as a log level.
    ///
    /// [`Status::Good`] | [`Status::Normal`] => [`tracing::info`]
    ///
    /// [`Status::Warn`] => [`tracing::warn`]
    ///
    /// [`Status::Bad`] | [`Status::Critical`] => [`tracing::error`]
    /// ```
    /// let my_status = halogen::Status::Warn;
    /// // logs at the warn level
    /// my_status.trace(format_args!("Time elapsed: {} seconds", 6));
    /// ```
    #[cfg(feature = "tracing")]
    pub fn trace(&self, message: std::fmt::Arguments<'_>) {
        match self {
            Self::Good | Self::Normal => tracing::info!(message),
            Self::Warn => tracing::warn!(message),
            Self::Bad | Self::Critical => tracing::error!(message),
        }
    }
}
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Good => "good",
            Self::Normal => "normal",
            Self::Warn => "warn",
            Self::Bad => "bad",
            Self::Critical => "critical",
        }
        .fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::imports::{from_string, to_string_pretty};

    #[test]
    fn deserialize_variants() {
        let mut variants = HashMap::new();
        variants.insert(
            "string".to_owned(),
            Variant::String("Hello world!".to_owned()),
        );
        variants.insert("path".to_owned(), Variant::Path(PathBuf::from("/usr/bin")));
        variants.insert("bool".to_owned(), Variant::Bool(true));
        variants.insert("signed int".to_owned(), Variant::Iint(-673485));
        variants.insert("unsigned_int".to_owned(), Variant::Uint(678656397));
        variants.insert("float".to_owned(), Variant::Float(-3778.489));

        variants.insert(
            "vector".to_owned(),
            Variant::Vec(vec![
                Box::new(Variant::Iint(-785)),
                Box::new(Variant::String("Hello World".to_owned())),
            ]),
        );

        let mut map = HashMap::new();
        map.insert(
            "name".to_owned(),
            Box::new(Variant::String("Drew".to_owned())),
        );
        map.insert("age".to_owned(), Box::new(Variant::Uint(32)));

        variants.insert("map".to_owned(), Variant::Map(map));

        let json = to_string_pretty(&variants).unwrap();

        let from_json: HashMap<String, Variant> = from_string(&json).unwrap();

        assert_eq!(from_json, variants);
    }
}
