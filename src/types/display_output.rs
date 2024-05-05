use std::sync::atomic::{AtomicU8, Ordering};

use crate::prelude::{error, Arc, Deserialize, Serialize};

/// Content that can be printed by the frontend.
///
/// To use this, impl `Into<DisplayOutput>` for `T`.
///
/// TODO: Finalize stuff required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayOutput {
    pub inner: String,
}

/// The type that I use for the module ID
pub type ModuleId = u8;
