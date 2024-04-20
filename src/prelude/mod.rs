mod imports;

pub(crate) use imports::*;

/// The default Result type for this crate. Not very descriptive.
pub type R<T> = Result<T, color_eyre::Report>;
