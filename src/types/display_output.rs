use std::sync::atomic::{AtomicU8, Ordering};

use crate::prelude::{error, Arc, Deserialize, Serialize};

/// Content that can be sent to the frontend.
///
/// TODO: Finalize stuff required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleData {
    pub inner: String,
}

/// The type that I use for the module ID
pub type ModuleId = u8;

/// A struct that will create new [`ModuleId`]s, it is basically a wrapper around an append-only integer.
///
/// Create this using `ModuleIdCreator::default()`
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ModuleIdCreator(ModuleId);
impl ModuleIdCreator {
    #[inline]
    pub fn create(&mut self) -> ModuleId {
        self.0 += 1;
        self.0
    }
}
