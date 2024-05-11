use crate::modules::{self, ModuleData, ModuleType, ModuleYield};
use crate::prelude::*;

// static BACKEND: OnceCell<Backend> = OnceCell::new();

// /// Get a reference to the backend information
// #[inline]
// pub fn get_backend() -> BackendResult<&'static Backend> {
//     BACKEND.get().ok_or(BackendError::Uninit)
// }

// /// Initialize the public backend, for use in initialization. This should only be called once!
// pub(super) fn initialize_backend(data: Vec<ModuleYield>) -> BackendResult<()> {
//     let me = Backend {
//         module_data: data
//             .into_iter()
//             .map(|d| (d.id, d))
//             .collect::<AHashMap<_, _>>(),
//     };

//     BACKEND
//         .set(me)
//         .map_err(|_| BackendError::DoubleInitialization)?;

//     Ok(())
// }

pub(super) struct Backend {
    /// The type of each module, the index of the module data in the map is the id of the corresponding module.
    modules: AHashMap<ModuleId, ModuleYield>,
    frontend_channel: mpsc::Sender<ModuleData>,
}
impl Backend {
    #[instrument(level = "trace", skip(self))]
    pub fn get_module_by_id<'d>(&'d self, id: &'d ModuleId) -> Option<&'d ModuleYield> {
        let data = self.modules.get(id);
        debug_assert!(data.is_some());

        data
    }

    /// This function is only used once in initialization!
    pub fn new(
        mut yielded_modules: Vec<ModuleYield>,
        frontend_channel: mpsc::Sender<ModuleData>,
    ) -> R<()> {
        // I might as well implement sorting myself, as I would have had to ensure each module
        // was at the correct index anyways
        let mut modules = Vec::with_capacity(yielded_modules.len());

        while let Some(module) = yielded_modules.pop() {
            
        }

        todo!();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BackendError {
    #[error("Not initialized")]
    Uninit,
    #[error("Double initialization!")]
    DoubleInitialization,
    #[error("Invalid ID: {0}")]
    InvalidId(ModuleId),
    #[error("Tried to send event data to a static module!")]
    SendStaticModule,
}
pub type BackendResult<T> = std::result::Result<T, BackendError>;
