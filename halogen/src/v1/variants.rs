use serde::{Deserialize, Serialize};

use std::collections::HashMap;

/// A recognized type that can be sent through IPC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Variant {
    /// A literal string of text. Only meant to be used for Strings.
    ///
    /// Please use [`Variant::Other`] for custom data types.
    String(String),

    /// A boolean value, Accepts true/false, on/off, 1/0, case-insensitive
    Bool(bool),
    /// signed byte
    I8(i8),
    /// unsigned byte
    U8(u8),
    /// signed integer
    Iint(isize),
    /// Unsigned integer
    Uint(usize),
    /// Single-precision floating point number
    Float(f32),
    /// Double-precision floating point number
    Double(f64),

    /// Multiple other Variants in a single-dimensional vector
    Vector(Vec<Box<Variant>>),
    /// Multiple other variants in a key-value map. The keys can only be Strings.
    Dict(HashMap<String, Box<Variant>>),

    /// Another data type, possibly serialized as a String.
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_variants() {}
}
