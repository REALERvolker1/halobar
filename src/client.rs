use crate::{
    modules::{ModuleData, RequestField},
    prelude::*,
};

/// The different states for a request
pub enum Request {
    Request(RequestField),
    Fulfilled(ModuleData),
    Error(ProviderError),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ProviderError {
    #[error("The field you provided is not a field this module supports: {0:?}")]
    InvalidField(RequestField),
    #[error("Error while getting the data")]
    QueryError,
    #[error("Invalid data type")]
    InvalidType,
}
/// A request for some data from a backend data provider module.
///
/// Each module sends a single request to each backend info provider it needs.
#[derive(Debug, Clone)]
pub struct DataRequest {
    pub id: ModuleId,
    pub data_fields: AHashMap<String, Option<ModuleData>>,
}
