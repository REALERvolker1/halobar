use crate::prelude::{Deserialize, Serialize};

type ModuleIdInteger = u8;

/// The type that I use for the module ID
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Display,
    derive_more::AsRef,
    Deserialize,
    Serialize,
)]
pub struct ModuleId(ModuleIdInteger);
impl ModuleId {
    /// Create a new ModuleId
    pub fn new(creator: &mut ModuleIdFactory) -> Self {
        creator.create()
    }

    /// Create a new ModuleId directly from the raw integer.
    pub unsafe fn new_unchecked(id: ModuleIdInteger) -> Self {
        Self(id)
    }

    /// Get the inner ID
    pub fn get(&self) -> ModuleIdInteger {
        self.0
    }

    /// Set this module's ID to an integer without checking if it is out of bounds or not.
    pub unsafe fn set_unchecked(&mut self, value: ModuleIdInteger) -> ModuleIdInteger {
        let old = self.0;
        self.0 = value;

        old
    }
}

/// A struct that will create new [`ModuleId`]s, it is basically a wrapper around an append-only integer.
///
/// Create this using `ModuleIdCreator::default()`
#[derive(Debug, PartialEq, Eq)]
pub struct ModuleIdFactory(ModuleIdInteger);
impl ModuleIdFactory {
    pub fn new() -> Self {
        Self(0)
    }
    pub fn create(&mut self) -> ModuleId {
        self.0 += 1;
        ModuleId(self.0)
    }
}
