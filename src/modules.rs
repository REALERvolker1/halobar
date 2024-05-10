pub mod time;
use tokio::runtime::Runtime;

use crate::prelude::*;

config_struct! {
    [Modules]
    // @conf network: network => Net,
    start_timeout_seconds: u64 = 5,
}

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
pub trait BackendModule: Sized + Send {
    /// The type of input that the module requires to create a new instance,
    /// including any type of config that the module requires for user customization.
    type Input;
    /// The type of module this is.
    const MODULE_TYPE: ModuleType;
    /// The function that runs this module. Consider this function blocking.
    ///
    /// Important: If it is a oneshot with no events, please specify! If it has to receive events, make it a loop.
    ///
    /// If it was expected to return, it will return `Ok(true)`. A bool value of `false` indicates it was supposed to run forever.
    async fn run(
        runtime: Arc<Runtime>,
        input: Self::Input,
        yield_sender: Arc<mpsc::UnboundedSender<(OutputType, ModuleType)>>,
    ) -> R<bool>;
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
#[derive(strum_macros::EnumDiscriminants)]
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

#[derive(Debug, strum_macros::Display, Serialize, Deserialize)]
pub enum ModuleData {
    Time(String),
}
