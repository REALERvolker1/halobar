use crate::{modules::ModuleData, prelude::*};

/// A request for some data from a backend data provider module.
///
/// Each module sends a single request to each backend info provider it needs.
#[derive(Debug, Clone)]
pub struct DataRequest {
    pub id: ModuleId,
    pub data_fields: AHashMap<String, Option<ModuleData>>,
}
