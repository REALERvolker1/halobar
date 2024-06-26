use serde::{Deserialize, Serialize};
use std::{convert::Infallible, mem::take, str::FromStr};

mod error;
pub use error::FormatStrError;
mod parser;
pub use parser::parse;
mod halotype;
pub use halotype::*;

/// A formatter struct whose keys correspond to variables in the format segments.
///
/// ```
/// use halobar_config::fmt::*;
///
/// #[derive(Debug, Default)]
/// pub struct Data {
///     value: u8,
///     other_value: bool,
/// }
///
/// pub struct DataFormatter {
///     data: Data,
///     segments: FmtSegmentVec,
///     fn_table: FnTable<Data, 2>,
/// }
/// impl DataFormatter {
///     pub fn new(initial_data: Data, segments: FmtSegmentVec) -> Self {
///         Self {
///             data: initial_data,
///             segments,
///             fn_table: FnTable([
///                 ("value", |d| {
///                     if d.value > 0 {
///                         return Some(d.value.to_string());
///                     }
///                     None
///                 }),
///                 ("other_value", |d| {
///                     if d.other_value {
///                         return Some(d.other_value.to_string());
///                     }
///                     None
///                 }),
///             ]),
///         }
///     }
/// }
///
/// impl HaloFormatter<2> for DataFormatter {
///     type Data = Data;
///     fn fn_table<'a>(&'a self) -> FnTable<Self::Data, 2> {
///         self.fn_table.copy()
///     }
///     fn current_data<'a>(&'a self) -> &'a Self::Data {
///         &self.data
///     }
///     fn set_data(&mut self, data: Self::Data) {
///         self.data = data
///     }
///     fn segments<'s>(&'s self) -> FmtSegments<'s> {
///         self.segments.segments()
///     }
///     fn default_format_str() -> FormatStr {
///         "Value {value?is $ percent:is empty}, other_value {other_value?is true:was not true}"
///             .to_owned()
///             .into()
///     }
/// }
///
/// fn main() {
///     let format_string = DataFormatter::default_format_str().parse().unwrap();
///     let mut formatter = DataFormatter::new(Data::default(), format_string);
///
///     let false_formatted = formatter.format().unwrap();
///
///     assert_eq!(false_formatted, "Value is empty, other_value was not true");
///
///     let new_data = Data {
///         value: 9,
///         other_value: true,
///     };
///     formatter.set_data(new_data);
///
///     let true_formatted = formatter.format().unwrap();
///     assert_eq!(true_formatted, "Value is 9 percent, other_value is true");
/// }
/// ```
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
    #[tracing::instrument(level = "trace", skip_all)]
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

        let mut out = String::with_capacity(segments.min_len);

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
    pub fn copy(&self) -> Self {
        Self(self.0)
    }
}

/// A simple formatter that formats any simple value that implements Display.
///
/// This takes any kind of [`FmtSegmentVec`] and replaces each truthy variable
/// occurrence with the data from its [`std::fmt::Display`] impl indiscriminately.
///
///
pub struct HaloDisplayFormatter<T: Truthy> {
    data: T,
    segvec: FmtSegmentVec,
}
impl<T: Truthy> HaloDisplayFormatter<T> {
    /// Format this segmented string. This is relatively expensive.
    pub fn format(&self) -> String {
        let mut alloc = String::with_capacity(self.segvec.min_length);
        for segment in self.segvec.segments() {
            match segment {
                Segment::Literal(l) => alloc.push_str(l),
                Segment::Variable(v) => {
                    if self.data.is_truthy() {
                        alloc.push_str(&v.truthy(&self.data.to_string()))
                    } else {
                        alloc.push_str(&v.falsy)
                    }
                }
            }
        }

        alloc
    }
}

/// Make a struct that allows you to iterate over the keys of a struct.
/// ```
/// use halobar_config::key_struct;
/// key_struct! {
///     /// An example struct to demonstrate the `key_struct!` macro
///     #[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
///     pub struct Example {
///         /// With attribute
///         #[allow(dead_code)]
///         pub name: String,
///         age: usize,
///         /// The user id of our example
///         pub(crate) id: usize,
///     }
/// }
///
/// fn main() {
///     let example: Example = Example {
///         name: String::from("Bob"),
///         age: 56,
///         id: 16,
///     };
///
///     assert_eq!(Example::KEYS.join(", "), "name, age, id");
///     assert_eq!(Example::NUM_KEYS, 3);
///
///     let bob_age_reference: &usize = example.get_age();
///     // Due to the declarative macro implementation, it always returns a reference, even for types implementing `Copy`.
///     let bob_age: usize = *bob_age_reference;
///
///     assert_eq!(bob_age, 56);
/// }
/// ```
#[macro_export]
macro_rules! key_struct {
    ( $( #[$struct_meta:meta] )* $struct_vis:vis struct $struct:ident { $( $( #[$field_meta:meta] )* $field_vis:vis $field:ident: $field_ty:ty ),+$(,)? } ) => {
        $( #[$struct_meta] )*
        $struct_vis struct $struct {
            $(
                $( #[$field_meta] )?
                $field_vis $field: $field_ty,
            )+
        }
        impl $struct {
            /// The number of keys in this struct
            pub const NUM_KEYS: usize = $crate::key_struct![_internal_key_count $( $field )+];
            /// An array that contains all the keys of the struct.
            pub const KEYS: [&'static str; Self::NUM_KEYS] = [$( stringify!($field) ),+];

            ::paste::paste! {
                $(
                    #[doc = "Get a reference to `" $struct "::" $field "`"]
                    $struct_vis fn [<get_ $field>]<'s>(&'s self) -> &'s $field_ty {
                        &self.$field
                    }
                )+
            }
        }
    };
    (_internal_replace_expr $thing:tt $sub:expr) => {$sub};
    (_internal_key_count $( $key:tt )*) => {
        0usize $( + $crate::key_struct!(_internal_replace_expr $key 1usize) )*
    };
}
