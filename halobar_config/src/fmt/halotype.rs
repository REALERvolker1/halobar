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
    /// A helpful field that allows you to cache the last output for greater performance.
    pub cached_value: Option<String>,
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
        parse(self.0.as_str())
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
impl<'a> FmtSegments<'a> {
    /// Format a data slice of `[(key, value), (k, v)...]`, consuming the [`FmtSegments`].
    ///
    /// This slice is collected into a new hashmap, where it is passed to [`FmtSegments::format_map`].
    pub fn format_data_slice<'b, D, I>(self, data: &[(&'b str, D)])
    where
        D: std::fmt::Display,
    {
        let data_iter = data.into_iter();
    }
    /// Format an entire data map of `"key": value`, consuming the [`FmtSegments`].
    ///
    /// Any variable whose key cannot be found in the map will be considered "falsy".
    /// Any variable whose key-value pair is in the map is considered "truthy".
    ///
    /// This creates a new String with the minimum memory allocation size specified in [`FmtSegments::min_len`].
    pub fn format_map<'b, S: AsRef<str>, H: std::hash::BuildHasher>(
        self,
        data: &std::collections::HashMap<&'b str, S, H>,
    ) -> String {
        let mut output = String::with_capacity(self.min_len);

        for segment in self {
            match segment {
                Segment::Literal(l) => output.push_str(l),
                Segment::Variable(var) => {
                    // I wanted to cache this but I couldn't figure out how to use the same hasher
                    match data.get(var.ident.as_str()) {
                        Some(data) => {
                            let data_string = var.truthy(data.as_ref());
                            output.push_str(&data_string);
                        }
                        None => {}
                    };
                }
            }
        }

        output
    }
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
    /// Create a new [`FmtSegmentVec`] from a string
    pub fn new(format_str: &str) -> Result<Self, FormatStrError> {
        parse(format_str)
    }
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
    /// Format an entire data map of `"key": value`, analogous to [`FmtSegments::format_map`], but non-consuming.
    ///
    /// Keys pointing to values of `Some<D>` will be considered truthy. Those pointing to values of `None` will be considered falsy.
    ///
    /// Any variable whose key cannot be found in the map will either use the cached value, or be considered falsy.
    ///
    /// This creates a new String with the minimum memory allocation size specified in the [`FmtSegmentVec::min_length`] property.
    pub fn format_map<'b, S: AsRef<str>, H: std::hash::BuildHasher>(
        &mut self,
        data: &std::collections::HashMap<&'b str, Option<S>, H>,
    ) -> String {
        let mut output = String::with_capacity(self.min_length);

        for segment in self.inner.iter_mut() {
            match segment {
                Segment::Literal(l) => output.push_str(l.as_str()),
                Segment::Variable(variable) => {
                    let query = data.get(variable.ident.as_str());

                    match query {
                        Some(maybe_value) => match maybe_value {
                            Some(value) => {
                                let truthy = variable.truthy(value.as_ref());
                                output.push_str(&truthy);

                                variable.cached_value.replace(truthy);
                            }

                            None => {
                                output.push_str(&variable.falsy);
                                // Note to self: This only happens if the provided value was None.
                                // It does not happen when getting the cached value.
                                variable.cached_value.take();
                            }
                        },

                        None => match variable.cached_value {
                            Some(ref cached) => output.push_str(cached),
                            None => output.push_str(&variable.falsy),
                        },
                    }
                }
            }
        }

        output
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
