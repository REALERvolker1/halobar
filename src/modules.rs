pub mod time;

use crate::{client::DataRequest, prelude::*, to_frontend::FrontendSender};
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
pub trait ModuleDataProvider: Sized + Send {
    /// The type of input that the module requires to create a new instance,
    /// including any type of config that the module requires for user customization.
    ///
    /// This must not have any required fields!!!
    ///
    /// This input is cloned before the module is created, so that duplicate modules/bars are not broken.
    /// The module is responsible for ensuring this is not too expensive.
    type ServerConfig: Clone;

    /// Any initialization required. Set up listeners, etc. This is step 1 of 3.
    ///
    /// Each module added may request new data to be monitored, in that case, it is
    /// the responsibility of the data provider to ensure these requests are met.
    async fn init(config: Self::ServerConfig, interface: ProviderData) -> R<Self>;

    /// Process requests for data at init time.
    ///
    /// This is step 2 of 3
    async fn process_data_requests(&mut self, requests: Vec<&mut DataRequest>) -> R<()>;

    /// Run this module (if it is a subscription provider)
    ///
    /// This is step 3 of 3
    async fn run(self) -> !;

    /// Create module data with this backend module's type.
    ///
    /// This is a shortcut meant to make stuff easier.
    fn module_data(content: Data) -> ModuleData {
        ModuleData {
            specific_target: None,
            content,
        }
    }
}

pub struct ProviderData {
    pub data_sender: FrontendSender<ModuleData>,
    pub request_receiver: mpsc::Receiver<DataRequest>,
}

/// All the data that is yielded from each module's `run()` function.
///
/// This is required to tie it to the frontend.
pub struct ModuleYield {
    pub id: ModuleId,
    pub initial_data: ModuleData,
    pub subscription: Option<mpsc::UnboundedReceiver<ModuleData>>,
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

// /// The type of module this is. This should contain every single different type of module.
// #[derive(
//     Debug,
//     Clone,
//     PartialEq,
//     Eq,
//     PartialOrd,
//     Ord,
//     Hash,
//     Serialize,
//     Deserialize,
//     strum_macros::Display,
//     strum_macros::AsRefStr,
//     strum_macros::EnumString,
// )]
// #[strum(serialize_all = "kebab-case")]
// #[serde(rename_all = "kebab-case")]
// pub enum ModuleType {
//     Time,
//     Custom,
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data {
    Time(time::TimeData),
}

/// Content that can be sent to the frontend.
///
/// TODO: Finalize stuff required.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleData {
    pub specific_target: Option<ModuleId>,
    pub content: Data,
    // pub module_type: ModuleType,
}
impl ModuleData {
    /// Create module data
    #[inline]
    pub fn new(content: Data) -> Self {
        Self {
            specific_target: None,
            content: content,
        }
    }
}
