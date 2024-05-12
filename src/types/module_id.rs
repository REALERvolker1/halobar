use crate::prelude::{debug, instrument, warn};

/// The raw integer type that the [`ModuleId`] contains
type ModuleIdInteger = u8;

/// The raw integer type that I am using for frontend module IDs.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display, derive_more::AsRef,
)]
pub struct ModuleId(ModuleIdInteger);
impl ModuleId {
    /// Get the inner integer
    pub fn get(&self) -> ModuleIdInteger {
        self.0
    }
}

/// An error that occured when generating a module ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub struct ModuleIdError;
impl std::fmt::Display for ModuleIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const FORMAT: &str = const_format::formatcp!(
            "Failed to generate new module ID, you have more than {}!",
            ModuleIdInteger::MAX
        );
        FORMAT.fmt(f)
    }
}

/// A generator that creates new [`ModuleId`]s.
///
/// # THIS MUST ONLY BE USED IN INITIALIZATION
pub struct ModuleIdFactory {
    inner: u8,
}
impl ModuleIdFactory {
    pub const fn new() -> Self {
        Self { inner: 0 }
    }
    /// Generate a new unique module ID.
    ///
    /// If the module ID is over the maximum of the underlying type,
    /// this will return an error.
    pub fn generate(&mut self) -> Result<ModuleId, ModuleIdError> {
        if self.inner == ModuleIdInteger::MAX {
            return Err(ModuleIdError);
        }

        self.inner += 1;
        Ok(ModuleId(self.inner))
    }
}
