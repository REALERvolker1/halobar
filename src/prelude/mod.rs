mod imports;

pub(crate) use imports::*;

/// The default Result type for this crate. Not very descriptive.
pub type R<T> = Result<T, color_eyre::Report>;

/// A shortcut for returning from an async block with an error type
///
/// - With no args like `rok![]` it returns (), or an eyre report.
/// - With a single arg, it returns the arg or an eyre report.
/// - With both a return and an error type, it returns either the return or the error type.
#[macro_export]
macro_rules! rok {
    () => {
        Ok::<_, $crate::prelude::Report>(())
    };
    ($return:expr) => {
        Ok::<_, $crate::prelude::Report>($return)
    };
    ($return:expr, $err:ty) => {
        Ok::<_, $err>($return)
    };
}
pub use rok;
