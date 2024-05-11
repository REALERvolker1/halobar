pub mod time;

use crate::prelude::*;
use std::cmp::Ordering;

/// A helper to make dbus proxy modules
#[macro_export]
macro_rules! proxy {
    ($conn:expr, $proxy:ty) => {
        <$proxy>::builder($conn)
            .cache_properties(::zbus::proxy::CacheProperties::No)
            .build()
    };
}
pub use proxy;

/// A module that can be used in the backend to provide data.
///
/// Multi-instance modules are not supported yet, but modules must always assume that there could
/// potentially be duplicates -- I don't know if I will ever add support, but I would like to keep the possibility open.
pub trait BackendModule: Sized + Send {
    /// The type of input that the module requires to create a new instance,
    /// including any type of config that the module requires for user customization.
    ///
    /// This must not have any required fields!!!
    ///
    /// This input is cloned before the module is created, so that duplicate modules/bars are not broken.
    /// The module is responsible for ensuring this is not too expensive.
    type Input: Clone;

    /// The type of module this is.
    const MODULE_TYPE: ModuleType;

    /// The function that runs this module. Consider this function blocking.
    ///
    /// Important: If it is a oneshot with no events, please specify! If it has to receive events, make it a loop.
    ///
    /// If it was expected to return, it will return `Ok(true)`. A bool value of `false` indicates it was supposed to run forever.
    async fn run(
        module_id: ModuleId,
        input: Self::Input,
        yield_sender: Arc<mpsc::UnboundedSender<ModuleYield>>,
    ) -> R<bool>;

    /// Create module data with this backend module's type.
    ///
    /// This is a shortcut meant to make stuff easier.
    fn module_data(data: String) -> ModuleData {
        ModuleData {
            format: data,
            module_type: Self::MODULE_TYPE,
        }
    }
}

/// All the data that is yielded from each module's `run()` function.
///
/// This is required to tie it to the frontend.
pub struct ModuleYield {
    pub id: ModuleId,
    pub data_output: OutputType,
    pub module_type: ModuleType,
}
impl ModuleYield {
    /// Compare two yields to each other. Used in the initializer functions
    pub fn id_ordering(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

/// A specific requirement that the module needs to work properly
#[derive(
    Debug, PartialEq, Eq, strum_macros::EnumTryAs, Serialize, Deserialize, strum_macros::Display,
)]
pub enum ModuleRequirement {
    SystemDbus,
    SessionDbus,
}

/// The type of module that this is. This determines a lot about how it is run.
#[derive(strum_macros::EnumDiscriminants, strum_macros::EnumTryAs, derive_more::From)]
pub enum OutputType {
    /// The module returns a constant through its channel on start, and is not run.
    OneShot(ModuleData),
    /// The module runs in a loop, pushing changes through its channel. The run function should never exit.
    Loop(BiChannel<Event, ModuleData>),
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
    Custom,
}

/// Content that can be sent to the frontend.
///
/// TODO: Finalize stuff required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleData {
    pub format: String,
    pub module_type: ModuleType,
}
impl ModuleData {
    /// Create module data
    #[inline]
    pub const fn new(module_type: ModuleType, format: String) -> Self {
        Self {
            format,
            module_type,
        }
    }
}
