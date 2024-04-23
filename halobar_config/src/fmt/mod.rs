use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, convert::Infallible, mem::take, str::FromStr};

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
pub trait HaloFormatter<const N: usize> {
    /// The type of data that this formatter will format
    type Data;
    /// Get the fields of the required data type -- the possible variable placeholders
    fn fn_table<'a>(&'a self) -> FnTable<Self::Data, N>;
    /// Get the current segment iterator
    fn segments<'s>(&'s self) -> FmtSegments<'s>;
    /// Get a sane default format str for this variable
    fn default_format_str() -> FormatStr;
    /// Extract the current data
    fn current_data<'a>(&'a self) -> &'a Self::Data;
    /// Replace the current data with new data
    fn set_data(&mut self, data: Self::Data);
    /// Get a map of the fields of this struct that are contained in the [`FmtSegments`].
    /// This is helpful for ignoring fields that you do not need to query, for example.
    ///
    /// Implementation detail: This does not have to be manually implemented.
    fn variable_map<'b>(&self) -> Result<Vec<&'static str>, FormatStrError> {
        let keys = self.fn_table().0.map(|(k, _)| k);
        let mut map = Vec::new();

        let variables = self.segments().filter_map(|s| match s {
            Segment::Literal(_) => None,
            Segment::Variable(v) => Some(&v.ident),
        });

        for var in variables {
            let varstr = var.as_str();
            for key in keys {
                if varstr == key {
                    map.push(key)
                }
            }
        }

        Ok(map)
    }
    /// Parse some segments, determining what to print. This takes data and determines how it should print.
    ///
    /// Implementation detail: This does not have to be manually implemented.
    fn format(&self) -> Result<String, FormatStrError> {
        let segments = self.segments();
        let fn_table = self.fn_table();
        let data = self.current_data();

        let mut out = String::new();

        for segment in segments {
            match segment {
                Segment::Literal(l) => out.push_str(l),
                Segment::Variable(v) => {
                    for (key, function) in fn_table.0 {
                        if v.ident == *key {
                            if let Some(s) = function(data) {
                                out.push_str(&v.truthy(&s))
                            } else {
                                out.push_str(&v.falsy)
                            }
                        }
                    }
                }
            }
        }

        Ok(out)
    }
}

/// An array of keys and functions used for the formatter inner function input
#[derive(Debug)]
pub struct FnTable<T, const N: usize>(pub [(&'static str, fn(&T) -> Option<String>); N]);
impl<T, const N: usize> FnTable<T, N> {
    #[inline(always)]
    fn copy(&self) -> Self {
        Self(self.0)
    }
}

#[derive(Debug, Default)]
pub struct Data {
    value: u8,
    other_value: bool,
}

pub struct DataFormatter {
    data: Data,
    segments: FmtSegmentVec,
    fn_table: FnTable<Data, 2>,
}
impl DataFormatter {
    pub fn new(initial_data: Data, segments: FmtSegmentVec) -> Self {
        Self {
            data: initial_data,
            segments,
            fn_table: FnTable([
                ("value", |d| {
                    if d.value > 0 {
                        return Some(d.value.to_string());
                    }
                    None
                }),
                ("other_value", |d| {
                    if d.other_value {
                        return Some(d.other_value.to_string());
                    }
                    None
                }),
            ]),
        }
    }
}

impl HaloFormatter<2> for DataFormatter {
    type Data = Data;
    fn fn_table<'a>(&'a self) -> FnTable<Self::Data, 2> {
        self.fn_table.copy()
    }
    fn current_data<'a>(&'a self) -> &'a Self::Data {
        &self.data
    }
    fn set_data(&mut self, data: Self::Data) {
        self.data = data
    }
    fn segments<'s>(&'s self) -> FmtSegments<'s> {
        self.segments.segments()
    }
    fn default_format_str() -> FormatStr {
        "Value {value?is $ percent:is empty}, other_value {other_value?is true:was not true}"
            .to_owned()
            .into()
    }
}

fn main() {
    let format_string = DataFormatter::default_format_str().parse().unwrap();
    let mut formatter = DataFormatter::new(Data::default(), format_string);

    let false_formatted = formatter.format().unwrap();

    let new_data = Data {
        value: 9,
        other_value: true,
    };
    formatter.set_data(new_data);

    let true_formatted = formatter.format().unwrap();
}

// FnTable([("value", |s, v, d| {}), ("other_value", |s, v, d| {})])
fn yuerkgbhk() {
    panic!("hi");
}
