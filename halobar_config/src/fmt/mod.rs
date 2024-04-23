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
pub struct FmtSegmentVec {
    inner: Vec<Segment>,
    min_length: usize,
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
    /// Just here so the string doesn't realloc a ton when printing
    min_len: usize,
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

// /// The inner function of [`HaloFormatter::format`]
// #[macro_export]
// macro_rules! haloformatter_format {
//     ($segments:expr; $( $key:ident: $determine_truth:expr => $inner_value ),+$(,)?) => {
//             let mut out = String::with_capacity($segments.min_len);

//             for segment in $segments {
//                 match segment {
//                     Segment::Literal(l) => out.push_str(l),
//                     Segment::Variable(v) => match v.ident.as_str() {
//                         $(
//                             stringify!($key) => {
//                                 let truthy = $determine_truth;
//                                 out.push_str(truthy);
//                             }
//                         )+
//                     },
//                 }
//             }

//             out
//     };
// }

/// A formatter struct whose keys correspond to variables in the format segments.
pub trait HaloFormatter {
    /// The type of data that this formatter will format
    type Data;
    /// Get the fields of the required data type -- the possible variable placeholders
    fn fields() -> &'static [&'static str];
    /// Get a map of the fields of this struct that are contained in the [`FmtSegments`].
    /// Each key corresponds to a boolean that denotes if the field is contained within the Segments.
    ///
    /// Implementation detail: This does not have to be manually implemented.
    /// ```
    /// ```
    fn variable_map<'b>(segments: FmtSegments<'b>) -> Result<Vec<&'static str>, FormatStrError> {
        let keys = Self::fields();
        let mut map = Vec::new();

        let variables = segments.filter_map(|s| match s {
            Segment::Literal(_) => None,
            Segment::Variable(v) => Some(&v.ident),
        });

        for var in variables {
            let varstr = var.as_str();
            for key in keys {
                if varstr == *key {
                    map.push(*key)
                }
            }
        }

        Ok(map)
    }
    /// Parse some segments, determining what to print. This takes data and determines how it should print.
    fn format(&self, data: Self::Data, segments: FmtSegments) -> Result<String, FormatStrError>;
    /// Get a sane default format str for this variable
    fn default_format_str() -> FormatStr;
}

// pub struct Data {
//     value: u8,
// }

// pub struct DataFormatter {
//     pub value: Data,
//     segments: FmtSegmentVec,
// }
// impl HaloFormatter for DataFormatter {
//     type Data = Data;
//     fn fields() -> &'static [&'static str] {
//         &["value"]
//     }
//     fn default_format_str() -> FormatStr {
//         "Value {value?is $ percent:is empty}".to_owned().into()
//     }
//     fn format(&self, data: Self::Data, segments: FmtSegments) -> Result<String, FormatStrError> {
//         haloformatter_format! {
//             segments;
//             value: {
//                 if data.value > 0 {

//                 }
//             },
//         }
//     }
// }
