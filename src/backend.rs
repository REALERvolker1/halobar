use futures_util::future::select_all;

use crate::modules::{self, ModuleData, ModuleType, ModuleYield};
use crate::prelude::*;

static BACKEND: OnceCell<Backend> = OnceCell::new();

/// Get a reference to the backend information
#[inline]
pub fn get_backend() -> BackendResult<&'static Backend> {
    BACKEND.get().ok_or(BackendError::Uninit)
}

pub struct Backend {
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

    /// This function is only used once in initialization! It will error if you use it multiple times.
    ///
    /// It sets the global once_cell `BACKEND` which you can then get with [`get_backend`].
    #[instrument(level = "debug", skip_all)]
    pub(crate) fn init(
        mut yielded_modules: Vec<ModuleYield>,
    ) -> BackendResult<BiChannel<EventData, ModuleData>> {
        if BACKEND.get().is_some() {
            return Err(BackendError::AlreadyInit);
        }

        let max_id = yielded_modules
            .iter()
            .map(|m| m.id.get())
            .max()
            .ok_or_else(|| BackendError::NoModules)?;

        let mut modules = (0..=max_id).map(|_| None).collect::<Vec<_>>();

        // I might as well implement sorting myself,
        // because I would have had to ensure each module was at the correct index anyways
        while let Some(module) = yielded_modules.pop() {
            let index = module.id.get_usize();

            modules[index] = Some(module);
        }

        let (channel, frontend) = BiChannel::new(64, Some("MUX receiver"), Some("MUX sender"));

        let me = Self { modules, channel };
        BACKEND.set(me).map_err(|_| BackendError::AlreadyInit)?;

        Ok(frontend)
    }

    /// Initialize an eavesdropping future.
    /// This logs all messages from the backend at info level, and internal errors at warn level.
    ///
    /// It should not return.
    pub async fn eavesdrop(&self) -> BackendResult<()> {
        let mut receivers = self
            .modules
            .iter()
            .filter_map(|m| m.as_ref())
            .filter_map(|m| m.data_output.try_as_loop_ref())
            .map(|c| &c.receiver)
            .map(|r| async {
                loop {
                    match r.recv_async().await {
                        Ok(data) => info!("{data}"),
                        Err(e) => {
                            warn!("Eavesdrop error: {e}");
                            break;
                        }
                    }
                }
            })
            .collect::<FuturesUnordered<_>>();

        while let Some(()) = receivers.next().await {}

        // This entire struct could only be created if there were supposed to be modules listening.
        // if there are no modules, what is the point?
        Err(BackendError::NoModules)
    }

    // TODO: Replace in frontend listener function with iced message types
    // pub(super) async fn run_listener(mut self) -> R<()> {
    //     let sending_future = async {
    //         let receivers = self
    //             .modules
    //             .iter()
    //             .filter_map(|m| m.as_ref())
    //             .filter_map(|m| m.data_output.try_as_loop_ref())
    //             .map(|c| &c.receiver)
    //             .collect::<Vec<_>>();
    //         // .map(|channel| receiver.recv_async());

    //         loop {
    //             // select_all panics when the input iter is empty. Since that is desired behavior,
    //             // I do not do any bounds checking here.
    //             // This entire struct could only be created if there were supposed to be modules listening.
    //             let next_data_selection = select_all(receivers.iter().map(|r| r.recv_async()));

    //             select! { biased;
    //                 next_data = next_data_selection => {
    //                     match next_data {
    //                         (Ok(data), _, _) => {
    //                             self.channel.send(data).await;
    //                         }
    //                         (Err(e), _, _) => {

    //                         }
    //                     }

    //                 }
    //                 event_result = self.channel.receiver.recv_async() => {
    //                     let event = event_result?;
    //                     let module = match self.get_module_by_id(event.module.get()) {
    //                         Ok(m) => {
    //                             let channel = m.data_output.try_as_loop_ref().ok_or_else(|| BackendError::StaticModule)?;

    //                             channel.send(event).await;
    //                         }
    //                         Err(e) => {
    //                             warn!("Error getting module: {e}");
    //                         }
    //                     };
    //                 }
    //             }
    //         }

    //         // self.channel.sender

    //         Ok::<(), Report>(())
    //     };

    //     Ok(())
    // }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BackendError {
    #[error("Not initialized")]
    Uninit,
    #[error("Already initialized! (Did you initialize twice??)")]
    AlreadyInit,
    #[error("Invalid ID: {0}")]
    InvalidId(ModuleIdInteger),
    #[error("Dead module ID: {0}")]
    DeadModule(ModuleIdInteger),
    #[error("Tried to interface with a static module!")]
    StaticModule,
    #[error("No modules")]
    NoModules,
}
pub type BackendResult<T> = std::result::Result<T, BackendError>;
