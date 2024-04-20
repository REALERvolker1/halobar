/// API version 1
pub mod v1;

/// An internal library for json abstractions
pub(crate) mod json;

/// Use the current version's definitions
pub use v1::*;
