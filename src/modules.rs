pub mod time;
pub mod upower;

use crate::{
    client::{DataRequest, ProviderError, Request},
    prelude::*,
    to_frontend::FrontendSender,
};

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

    /// This is the entry point for the data provider. This initializes it with its config,
    /// its interface to the outside world, and a buffer that tells it what to watch for.
    ///
    /// It takes ownership of the data requests, fulfills them (or provides errors if it can't fulfull them),
    /// and then passes the request vector back out with initial data values so the frontend can initialize whatever asked for data.
    async fn main(
        config: Self::ServerConfig,
        requests: Vec<DataRequest>,
        yield_channel: mpsc::UnboundedSender<ModuleYield>,
    ) -> R<()>;
}

/// All the data that is yielded from each data provider.
///
/// It will provide a channel for events to be sent and data to be received if it
/// is a dynamic module.
///
/// Additionally, it will give the initial data request vector back to the frontend.
///
/// This is required to tie it to the frontend.
pub struct ModuleYield {
    pub subscription: Option<BiChannel<Event, ModuleData>>,
    pub fulfilled_requests: Vec<DataRequest>,
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum Data {
//     Time(time::TimeData),
// }

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
    pub const fn new(content: Data) -> Self {
        Self {
            specific_target: None,
            content,
        }
    }
}

macro_rules! data_enum {
    ($( [$module:ident] data_type: $( $data_type:ty ),+; request_field: $req_field_type:ty );+$(;)?) => {
        /// The type of module. Should be tiny and contain nothing
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
        pub enum ModuleType {
            $( $module ),+
        }

        /// The data a module can provide. This is an enum, with a branch for each
        /// data provider, and an inner tuple where the data is carried.
        #[derive(Debug, Clone, PartialEq)]
        pub enum Data {
            $( $module($( $data_type ),+) ),+
        }

        /// These are the fields that you can request. This is sent to the providers.
        #[derive(Debug, Clone, PartialEq)]
        pub enum RequestField {
            $( $module($req_field_type) ),+
        }
    };
}

data_enum! {
    [Time]
    data_type: time::TimeData;
    request_field: String;
    [Upower]
    data_type: upower::UpowerData;
    request_field: upower::UpowerDataDiscriminants;
}
