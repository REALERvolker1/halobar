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
    pub module: ModuleIdentity,
}

/// The type of module this is. This should contain every single different type of module.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    strum_macros::Display,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum ModuleType {
    Time,
    Network,
}

/// The type that I use for the module ID
pub type ModuleId = u8;

/// I want to make sure that no two ever get the same ID.
static NEXT_MODULE_ID: AtomicU8 = AtomicU8::new(0);

/// The unique identity of a single module. No two should be alike.
///
/// This wraps an Arc, so it is safe to clone.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ModuleIdentity {
    inner: Arc<ModuleIdentityInner>,
}
impl ModuleIdentity {
    /// Create a new Module identity. This has a slight synchronization cost.
    ///
    /// It will PANIC if there are more than the maximum amount allowed by the underlying integer.
    pub fn new(module_type: ModuleType) -> Self {
        const OVERFLOW_MSG: &str =
            "Failed to create a new module ID, you have reached the module limit!";

        let id = NEXT_MODULE_ID
            .fetch_update(Ordering::Release, Ordering::Acquire, |former| {
                if former == u8::MAX {
                    error!("{OVERFLOW_MSG}");
                    return None;
                }
                Some(former + 1)
            })
            // safety: Who would ever need more than 256 modules? lol
            .expect(OVERFLOW_MSG);

        Self {
            inner: Arc::new(ModuleIdentityInner { module_type, id }),
        }
    }
    /// Get a reference to this module's mod type
    #[inline]
    pub fn mod_type<'i>(&'i self) -> &'i ModuleType {
        &self.inner.module_type
    }
    /// Get a reference to this module's ID
    #[inline]
    pub fn id(&self) -> u8 {
        self.inner.id
    }
}
impl std::fmt::Debug for ModuleIdentity {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct ModuleIdentityInner {
    module_type: ModuleType,
    id: ModuleId,
}
impl PartialOrd for ModuleIdentityInner {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.module_type.partial_cmp(&other.module_type)
    }
}
impl Ord for ModuleIdentityInner {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.module_type.cmp(&other.module_type)
    }
}
