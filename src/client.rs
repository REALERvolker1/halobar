use crate::{
    modules::{ModuleData, RequestField},
    prelude::*,
};

/// The different states for a request
#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    Request(RequestField),
    Fulfilled(ModuleData),
    Error(ProviderError),
}
impl Request {
    pub fn resolve(&mut self, data: ModuleData) {
        *self = Self::Fulfilled(data)
    }
    pub fn reject(&mut self, error: ProviderError) {
        *self = Self::Error(error)
    }
    /// Return this as an error with [`ProviderError::InvalidRequest`].
    ///
    /// Clones and boxes the request internally.
    pub fn reject_invalid(&mut self) {
        let boxed_req = Box::new(self.clone());
        *self = Request::Error(ProviderError::InvalidRequest(boxed_req))
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ProviderError {
    #[error("The field you provided is not a field this module supports: {0:?}")]
    InvalidField(RequestField),
    #[error("Error while getting the data")]
    QueryError,
    #[error("Invalid data type")]
    InvalidType,
    #[error("Invalid request: {0:?}")]
    InvalidRequest(Box<Request>),
}
/// A request for some data from a backend data provider module.
///
/// Each module sends a single request to each backend info provider it needs.
#[derive(Debug, Clone)]
pub struct DataRequest {
    pub id: ModuleId,
    pub data_fields: Vec<Request>,
}
impl DataRequest {}
