use serde::{Deserialize, Serialize};
use std::{convert::Infallible, mem::take, str::FromStr};

mod error;
pub use error::FormatStrError;
mod parse;
pub use parse::parse;
mod halotype;
pub use halotype::*;

/// The inner representation of a fmt string.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FmtSegmentVec(Vec<Segment>);
impl FmtSegmentVec {
    /// Get the inner Vec of [`Segment`], consuming self
    #[inline(always)]
    pub fn to_vec(self) -> Vec<Segment> {
        self.0
    }
    /// Get a [`FmtSegments`] for this Vec, which allows for iteration.
    #[inline(always)]
    pub fn segments<'a>(&'a self) -> FmtSegments<'a> {
        FmtSegments {
            inner: self.0.as_slice(),
            current_idx: 0,
        }
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
    /// Parse this string into [`FmtSegmentVec`]
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
    inner: &'a [Segment],
    current_idx: usize,
}
impl<'a> Iterator for FmtSegments<'a> {
    type Item = &'a Segment;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.get(self.current_idx)?;
        self.current_idx += 1;
        Some(item)
    }
}

/// The inner representation of a var string.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Variable {
    /// The variable name as a String
    pub ident: String,
    /// These segments are printed in order, joined with the value.
    truthy: Vec<String>,
    /// The default "placeholder" value to display when there is no value
    pub falsy: String,
}
impl Variable {
    /// Get the correct string to show when the variable is truthy
    pub fn truthy(&self, value: &str) -> String {
        if self.truthy.is_empty() {
            return value.to_owned();
        }

        self.truthy.join(value)
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

/// An enum used internally. It is marked as public because it could be part of an error message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserState {
    /// Currently parsing a Literal
    Literal,
    /// Parsing the variable name segment
    VarIdent,
    /// Parsing the truthy segment
    VarTruthy,
    /// Parsing the falsy segment
    VarFalsy,
}

/// A formatter struct whose keys correspond to variables in the format segments.
pub trait HaloFormatter {
    /// The type of data that this formatter will format
    type Data;
    /// Get a map of the fields of this struct that are contained in the [`FmtSegments`].
    /// Each key corresponds to a boolean that denotes if the field is contained within the Segments.
    ///
    /// Implementation detail: You may use the [`variable_map`] function in the crate root to do most of the heavy lifting.
    /// ```
    /// ```
    fn variable_map<'b>(
        &self,
        segments: FmtSegments<'b>,
    ) -> Result<Vec<&'static str>, FormatStrError>;
    /// Parse some segments, determining what to print. This takes data and determines how it should print.
    fn format(&self, data: Self::Data, segments: FmtSegments) -> Result<String, FormatStrError>;
    /// Get a sane default format str for this variable
    fn default_format_str() -> FormatStr;
}

/// The internal [`HaloFormatter::variable_map`] function
pub fn variable_map<'b, const N: usize>(
    keys: [&'static str; N],
    segments: FmtSegments<'b>,
) -> Result<[(&'static str, bool); N], FormatStrError> {
    let mut map = keys.map(|k| (k, false));

    let variables = segments.filter_map(|s| match s {
        Segment::Literal(_) => None,
        Segment::Variable(v) => Some(&v.ident),
    });

    for var in variables {
        let varstr = var.as_str();
        map = map.map(|(k, b)| if k == varstr { (k, true) } else { (k, b) });
    }

    Ok(map)
}
