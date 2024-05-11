use crate::modules::{self, ModuleData, ModuleType, ModuleYield};
use crate::prelude::*;

static BACKEND: OnceCell<Backend> = OnceCell::new();

/// Get a reference to the backend information
#[inline]
pub fn get_backend() -> BackendResult<&'static Backend> {
    BACKEND.get().ok_or(BackendError::Uninit)
}

/// Initialize the public backend, for use in initialization. This should only be called once!
pub(super) fn initialize_backend(data: Vec<ModuleYield>) -> BackendResult<()> {
    let me = Backend {
        module_data: data
            .into_iter()
            .map(|d| (d.id, d))
            .collect::<AHashMap<_, _>>(),
    };

    BACKEND
        .set(me)
        .map_err(|_| BackendError::DoubleInitialization)?;

    Ok(())
}

pub struct Backend {
    module_data: AHashMap<ModuleId, ModuleYield>,
}
impl<'b> Backend {
    /// Send event data to a specific module.
    ///
    /// This returns true if it sent correctly, false if it did not send correctly,
    /// and an internal error if the message itself is invalid.
    pub async fn send_event(&'b self, event: Event, module_id: &ModuleId) -> BackendResult<bool> {
        let module = self
            .module_data
            .get(module_id)
            .ok_or_else(|| BackendError::InvalidId(*module_id))?;

        let channel = module
            .data_output
            .try_as_loop_ref()
            .ok_or_else(|| BackendError::SendStaticModule)?;

        let sent = channel.send(event).await;

        Ok(sent)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
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
