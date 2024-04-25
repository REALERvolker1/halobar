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

/// A message payload sent from either the server to the client, or client to the server
///
/// The Version information is sent as the first byte of the bytevec
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// The current status. Signifies the current state of things.
    pub status: Status,
    /// The type of sender that sent this message
    pub sender_type: Target,
    /// The name of this message
    pub identifier: String,
    // /// Inner data payload
    // pub data: HashMap<String, Variant>,
    /// The text to display (client)
    pub display: String,
}
impl Message {
    pub const VERSION: u8 = 1;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    /// Messages only sent to the server
    Server {
        display: Option<String>,
        percent: Option<u8>,
    },
    /// Messages that are sent to the client
    Client { event: Event },
}
impl Default for Target {
    fn default() -> Self {
        Self::Server {
            display: None,
            percent: None,
        }
    }
}

/// Different events that the bar listens to that are sent to any clients that request events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Click,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

/// A general-purpose status
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    derive_more::Display,
)]
pub enum Status {
    #[display(fmt = "good")]
    Good,
    #[default]
    #[display(fmt = "normal")]
    Normal,
    #[display(fmt = "warn")]
    Warn,
    #[display(fmt = "bad")]
    Bad,
    #[display(fmt = "critical")]
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
