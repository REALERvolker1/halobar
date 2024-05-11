use crate::modules::{self, ModuleData, ModuleType, ModuleYield};
use crate::prelude::*;

// static BACKEND: OnceCell<Backend> = OnceCell::new();

// /// Get a reference to the backend information
// #[inline]
// pub fn get_backend() -> BackendResult<&'static Backend> {
//     BACKEND.get().ok_or(BackendError::Uninit)
// }

pub(super) struct Backend {
    /// The type of each module, the index of the module data in the map is the id of the corresponding module.
    modules: Vec<Option<ModuleYield>>,
    channel: BiChannel<ModuleData, EventData>,
}
impl Backend {
    #[instrument(level = "trace", skip(self))]
    pub fn get_module_by_id<'d>(&'d self, id: ModuleIdInteger) -> BackendResult<&'d ModuleYield> {
        match self.modules.get(id as usize) {
            Some(Some(m)) => Ok(m),
            Some(None) => Err(BackendError::DeadModule(id)),
            None => Err(BackendError::InvalidId(id)),
        }
    }

    /// This function is only used once in initialization!
    #[instrument(level = "debug", skip_all)]
    pub fn new(
        mut yielded_modules: Vec<ModuleYield>,
    ) -> R<(Arc<Self>, BiChannel<EventData, ModuleData>)> {
        // I might as well implement sorting myself,
        // because I would have had to ensure each module was at the correct index anyways

        let max_id = yielded_modules
            .iter()
            .map(|m| m.id.get())
            .max()
            .ok_or_eyre("No backend modules provided!")?;

        let mut modules = (0..=max_id).map(|_| None).collect::<Vec<_>>();

        while let Some(module) = yielded_modules.pop() {
            let index = module.id.get_usize();

            modules[index] = Some(module);
        }

        let (channel, frontend) = BiChannel::new(32, Some("MUX receiver"), Some("MUX sender"));

        Ok((Arc::new(Self { modules, channel }), frontend))
    }

    pub(super) async fn run_listener(mut self) -> R<()> {
        // let receiving_future = async {
        //     while let Some(event) = self.channel.receiver.recv().await {
        //         let module = match self.get_module_by_id(event.module.get()) {
        //             Ok(m) => m,
        //             Err(e) => {
        //                 warn!("Error getting module: {e}");
        //                 continue;
        //             }
        //         };

        //         let channel = module
        //             .data_output
        //             .try_as_loop_ref()
        //             .ok_or_else(|| BackendError::SendStaticModule)?;

        //         channel.send(event).await;
        //     }

        //     Ok::<(), Report>(())
        // };

        let sending_future = async {
            let active_channels = self
                .modules
                .iter()
                .filter_map(|m| m.as_ref())
                .filter_map(|m| m.data_output.try_as_loop_ref());


            // self.channel.sender

            Ok::<(), Report>(())
        };

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BackendError {
    #[error("Not initialized")]
    Uninit,
    #[error("Invalid ID: {0}")]
    InvalidId(ModuleIdInteger),
    #[error("Dead module ID: {0}")]
    DeadModule(ModuleIdInteger),
    #[error("Tried to send event data to a static module!")]
    SendStaticModule,
}
pub type BackendResult<T> = std::result::Result<T, BackendError>;
