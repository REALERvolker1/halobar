use crate::imports::*;

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
    #[instrument(level = "trace", skip_all)]
    pub fn from_iterator<I: IntoIterator<Item = Self>>(vec: I) -> Self {
        Self::Vec(vec.into_iter().map(|v| Box::new(v)).collect())
    }
    /// Get a wholly owned Vec from this Variant, if it is a Vec type. Otherwise returns the provided Variant.
    #[instrument(level = "trace", skip_all)]
    pub fn unboxed_vec(self) -> Result<Vec<Variant>, Variant> {
        if let Self::Vec(v) = self {
            let unboxed = v.into_iter().map(|b| *b).collect();
            return Ok(unboxed);
        }
        Err(self)
    }
    /// Get a wholly owned hashmap out of this Variant. Only works for Map types, please see [`Variant::unboxed_vec`]
    #[instrument(level = "trace", skip_all)]
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
    version: u8,
    /// The current status. Signifies the current state of things.
    pub status: Status,
    /// The type of sender that sent this message
    sender_type: SenderType,
    /// The name of this message
    pub name: String,
    /// Inner data payload
    pub data: HashMap<String, Variant>,
}
impl Default for Message {
    fn default() -> Self {
        Self::new(
            Status::default(),
            SenderType::All,
            String::new(),
            HashMap::new(),
        )
    }
}
impl Message {
    // pub fn new(key: &str)
    /// Deserialize a message from raw json bytes.
    ///
    /// This requires a mutable slice because the simd-json crate will mutate the slice and I want this to be zerocopy.
    #[instrument(level = "trace", skip_all)]
    pub fn try_from_raw(json: &mut [u8]) -> Result<Self, crate::imports::Error> {
        crate::imports::from_bytes(json)
    }
    /// Create a new message directly. Not recommended to use -- instead, use the helper impls provided by dedicated interfaces.
    #[instrument(level = "debug", skip_all)]
    pub fn new<S: Into<String>>(
        status: Status,
        sender_type: SenderType,
        name: S,
        data: HashMap<String, Variant>,
    ) -> Self {
        Self {
            version: 1,
            status,
            sender_type,
            name: name.into(),
            data,
        }
    }
    /// Serialize this message into json fit to send to the socket.
    #[instrument(level = "trace", skip_all)]
    pub fn into_json(&self) -> Result<String, Error> {
        let out = json::to_string(self)?;
        Ok(out)
    }
    #[inline]
    pub const fn version(&self) -> u8 {
        self.version
    }
    #[inline]
    pub const fn sender_type(&self) -> SenderType {
        self.sender_type
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SenderType {
    /// Messages only interpreted by the server
    Server,
    /// Messages that are only interpreted by the client
    Client,
    /// Messages that are sent to everyone everywhere
    #[default]
    All,
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

    use crate::imports::{from_bytes, json::to_string};

    /// TODO: This test is outdated
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

        let mut json = to_string(&variants).unwrap();

        // safety: I know this is UTF-8
        let json_bytes = unsafe { json.as_bytes_mut() };
        let from_json: HashMap<String, Variant> = from_bytes(json_bytes).unwrap();

        assert_eq!(from_json, variants);
    }
}
