use super::*;

/// An internal enum made to determine what type of content should go next in a variable
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum VarContentType {
    /// Show the variable value
    #[default]
    Value,
    /// Just a literal string
    Literal(String),
}
impl VarContentType {
    #[inline]
    pub fn try_subst<'a>(&'a self, maybe_substitute: &'a str) -> &'a str {
        match self {
            Self::Value => maybe_substitute,
            Self::Literal(l) => l.as_str(),
        }
    }
}
impl From<String> for VarContentType {
    fn from(value: String) -> Self {
        VarContentType::Literal(value)
    }
}

/// The inner representation of a var string.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Variable {
    /// The variable name as a String
    pub ident: String,
    /// These segments are printed in order, joined with the value.
    pub truthy: Vec<VarContentType>,
    /// The default "placeholder" value to display when there is no value
    pub falsy: String,
    #[serde(skip)]
    pub(crate) content_position: VarContentType,
}
impl Variable {
    /// Get the correct string to show when the variable is truthy
    pub fn truthy(&self, value: &str) -> String {
        self.truthy
            .iter()
            .map(|t| t.try_subst(value))
            .collect::<Vec<_>>()
            .concat()
    }
}

/// An individual segment of a FormatVec
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Segment {
    /// A String to pass in, verbatim
    Literal(String),
    /// A variable, denoted with special syntax
    Variable(Variable),
}
impl Default for Segment {
    fn default() -> Self {
        Self::Literal(Default::default())
    }
}

/// A raw String that contains special syntax for formatting
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deserialize,
    Serialize,
    derive_more::Display,
    derive_more::From,
    derive_more::AsRef,
)]
pub struct FormatStr(String);
impl FormatStr {
    /// Parse this string into [`FmtSegmentVec`]. See [`parse`] for more info
    #[inline(always)]
    pub fn parse(self) -> Result<FmtSegmentVec, FormatStrError> {
        parse(self.0)
    }
    /// Get the internal string as a slice
    #[inline(always)]
    pub fn str<'a>(&'a self) -> &'a str {
        &self.0
    }
    /// Get the internal string, consuming
    #[inline(always)]
    pub fn string(self) -> String {
        self.0
    }
}
impl FromStr for FormatStr {
    type Err = Infallible;
    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

/// A borrowed FmtSegmentVec. Useful for copying.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FmtSegments<'a> {
    /// Just here so the string doesn't realloc a ton when printing
    pub min_len: usize,
    pub inner: &'a [Segment],
    pub current_idx: usize,
}
impl<'a> Iterator for FmtSegments<'a> {
    type Item = &'a Segment;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.get(self.current_idx)?;
        self.current_idx += 1;
        Some(item)
    }
}

/// The inner representation of a fmt string.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FmtSegmentVec {
    pub inner: Vec<Segment>,
    pub min_length: usize,
}
impl FmtSegmentVec {
    /// Get the inner Vec of [`Segment`], consuming self
    #[inline]
    pub fn to_vec(self) -> Vec<Segment> {
        self.inner
    }
    /// Get a [`FmtSegments`] for this Vec, which allows for iteration.
    #[inline]
    pub fn segments<'a>(&'a self) -> FmtSegments<'a> {
        FmtSegments {
            min_len: self.min_length,
            inner: self.inner.as_slice(),
            current_idx: 0,
        }
    }
}

/// Methods that a type can use to expedite formatting.
pub trait Truthy: std::fmt::Display {
    /// Determine if this variable is "truthy" or "falsy"
    fn is_truthy(&self) -> bool;
}

macro_rules! mass_impl {
    (@int $( $ty:ty ),+$(,)?) => {
        $(
            impl Truthy for $ty {
                #[inline(always)]
                fn is_truthy(&self) -> bool {
                    *self != 0
                }
            }
        )+
    };
    (@float $( $ty:ty ),+$(,)?) => {
        $(
            impl Truthy for $ty {
                #[inline(always)]
                fn is_truthy(&self) -> bool {
                    *self != 0.0
                }
            }
        )+
    };
    (@str $( $ty:ty ),+$(,)?) => {
        $(
            impl Truthy for $ty {
                #[inline(always)]
                fn is_truthy(&self) -> bool {
                    !self.is_empty()
                }
            }
        )+
    };
}

mass_impl![@int i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize];
mass_impl![@float f32, f64];
mass_impl![@str String, str];

impl Truthy for bool {
    #[inline(always)]
    fn is_truthy(&self) -> bool {
        *self
    }
}