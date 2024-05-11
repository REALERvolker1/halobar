use std::sync::atomic::Ordering;

use crate::prelude::{instrument, trace, warn, Arc};

/// The raw integer type that the [`ModuleId`] contains
pub type ModuleIdInteger = u8;
/// The atomic version of [`ModuleIdInteger`]
type AtomicModuleIdInteger = std::sync::atomic::AtomicU8;

static MODULE_ID_GEN: AtomicModuleIdInteger = AtomicModuleIdInteger::new(0);

#[instrument(level = "trace")]
fn generate_new_module_id() -> Option<ModuleIdInteger> {
    let generated = MODULE_ID_GEN
        .fetch_update(Ordering::Release, Ordering::Acquire, |current| {
            if current == ModuleIdInteger::MAX {
                return None;
            }

            let new = current + 1;
            Some(new)
        })
        .ok();

    match generated {
        Some(new) => trace!("Generated new module ID: {new}"),
        None => warn!(
            "Failed to generate new module ID, you have more than {}!",
            ModuleIdInteger::MAX
        ),
    }

    generated
}

/// The type that I use for the module ID. It contains an Arc, so it is relatively cheap to clone.
///
/// Several methods on this struct are unsafe -- not because they use unsafe rust,
/// but because using them could easily result in undefined behavior.
///
/// I am intentionally making this hard to use, because it is an essential part of the internal
/// message passing API and if you misuse it, you must be punished severely.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
pub struct ModuleId {
    inner: Arc<ModuleIdInteger>,
}
impl ModuleId {
    /// Create a new ModuleId. Very expensive synchronization, please do not run this in
    /// code that isn't initialization-related.
    pub fn try_new() -> Option<Self> {
        let integer = generate_new_module_id()?;
        // safety: We generated the inner value using the single intended method
        Some(unsafe { Self::new_unchecked(integer) })
    }

    /// Create a new ModuleId directly from the raw integer,
    /// without verifying it is not already taken by another module
    ///
    /// Safety: This technically uses no unsafe code internally,
    /// but the entire concept itself is inherently unsafe.
    pub unsafe fn new_unchecked(id: ModuleIdInteger) -> Self {
        Self {
            inner: Arc::new(id),
        }
    }

    /// Get the inner ID
    pub fn get(&self) -> ModuleIdInteger {
        *self.inner
    }

    /// Get this type as a usize, suitable for indexing operations
    pub fn get_usize(&self) -> usize {
        usize::from(self.get())
    }

    /// Set this module's ID to an integer without checking if it is out of bounds or not.
    ///
    /// Safety: This technically uses no unsafe code internally,
    /// but the entire concept itself is inherently unsafe.
    pub unsafe fn set_unchecked(&mut self, value: ModuleIdInteger) -> ModuleIdInteger {
        let old = self.get();
        let new = ModuleId::new_unchecked(value);
        *self = new;

        old
    }
}
impl AsRef<ModuleIdInteger> for ModuleId {
    fn as_ref(&self) -> &ModuleIdInteger {
        &self.inner
    }
}
impl std::fmt::Debug for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ModuleId").field(&self.get()).finish()
    }
}
