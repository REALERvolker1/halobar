use crate::prelude::ModuleId;
use halogen::Event;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventData {
    pub event: Event,
    pub module: ModuleId,
}
