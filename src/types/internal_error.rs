/// An error type used internally. This should never be constructed at runtime.
#[derive(
    Debug, crate::prelude::SmartDefault, Clone, Copy, derive_more::Display, derive_more::Error,
)]
#[display(fmt = "Internal error in module '{}': {}", module, message)]
pub struct InternalError {
    #[default("No message specified")]
    pub message: &'static str,
    #[default("Unspecified module!")]
    pub module: &'static str,
}
impl InternalError {
    pub const fn new(module: &'static str, message: &'static str) -> Self {
        Self { module, message }
    }
}

pub type InternalResult<T> = std::result::Result<T, InternalError>;
