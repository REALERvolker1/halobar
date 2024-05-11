use crate::modules::{self, BackendModule, ModuleType, ModuleYield};
use crate::prelude::*;
use tokio::runtime::Runtime;

#[inline]
pub async fn run(runtime: Arc<Runtime>, config: ModuleConfig) -> R<()> {
    let mut initializer = BackendInitializer::new(runtime.clone(), config).await?;

    // Spawn each module on a task -- they start running instantly!
    let mut handles = initializer.run().await?;

    // Get the yielded data from this function!
    let modules = initializer.receive_from_channels().await?;

    // TODO: Connect to frontend
    let frontend_channel = crate::backend::Backend::init(modules)?;

    let _eavesdrop_handle =
        runtime.spawn(async move { crate::backend::get_backend()?.eavesdrop().await });

    while let Some(res) = handles.next().await {
        let (mod_name, module_return) = match res {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to join task: {e}");
                continue;
            }
        };

        match module_return {
            Ok(true) => debug!("Module {} returned true", mod_name),
            Ok(false) => warn!("Module {} returned incorrectly!", mod_name),
            Err(e) => error!("Module {} returned error: {}", mod_name, e),
        }
    }

    warn!("Finished backend execution!");
    Ok(())
}

const DEFAULT_START_TIMEOUT_SECONDS: u64 = 5;

/// The main module config.
///
/// TODO: Implement multi-instance modules
#[derive(Debug, SmartDefault, Serialize, Deserialize)]
pub struct ModuleConfig {
    #[default(Some(DEFAULT_START_TIMEOUT_SECONDS))]
    pub start_timeout_seconds: Option<u64>,
    pub time: modules::time::TimeConfig,
}

struct BackendInitializer {
    runtime: Arc<Runtime>,
    receiver: mpsc::UnboundedReceiver<ModuleYield>,
    sender: Option<Arc<mpsc::UnboundedSender<ModuleYield>>>,
    /// I need this to be a hashmap because the listeners do not return in an ordered manner.
    expected_modules: AHashMap<ModuleId, ModuleType>,
    config: ModuleConfig,
}
impl BackendInitializer {
    /// Internal backend initializer creation function
    pub async fn new(runtime: Arc<Runtime>, config: ModuleConfig) -> R<Self> {
        // Each module must send a listener to this channel when they are ready to push data.
        // I made it this way because rust doesn't have generator functions, and I needed a way for functions to yield values
        // when I need them, and for them to just run as single instances. A bunch of dbus proxy lifetime stuff is involved there too.
        // It would have been messy to make a Module::new() and Module::run() thing when the run method would have just errored out instantly.
        let (sender, receiver) = mpsc::unbounded_channel();

        Ok(Self {
            runtime,
            receiver,
            sender: Some(Arc::new(sender)),
            expected_modules: AHashMap::new(),
            config,
        })
    }

    /// The second component of the runtime. This initializes modules and runs them in tokio tasks.
    ///
    /// It returns handles to each module's task.
    #[instrument(level = "trace", skip_all)]
    pub async fn run(
        &mut self,
    ) -> R<FuturesUnordered<tokio::task::JoinHandle<(&'static str, Result<bool, Report>)>>> {
        let handles = FuturesUnordered::new();
        let sender = self.sender.take().expect(
            "Runtime functions out of order! Backend initializer is missing its internal sender!",
        );

        // TODO: Make this a trait that all module configs must impl
        macro_rules! init_module {
            ($( [$mod_name:expr] module: $mod_path:ty, input: $input:expr ),+$(,)?) => {$({
                let yield_sender = Arc::clone(&sender);
                let input = $input;
                let id = ModuleId::try_new().expect(::const_format::concatcp!("Failed to create a module ID for module", stringify!($mod_name)));

                trace!("Initializing module {}:{}", $mod_name, id);
                self.expected_modules.insert(id.clone(), <$mod_path>::MODULE_TYPE);

                let handle = self.runtime.spawn(async move {
                    let module_return = <$mod_path>::run(id, input, yield_sender).await;
                    ($mod_name, module_return)
                });

                handles.push(handle);
            })+};
        }

        init_module! {
            ["time"]
            module: crate::modules::time::Time,
            input: self.config.time.clone(),
        }

        Ok(handles)
    }

    /// The third component of initialization.
    ///
    /// This waits for each module to return a listener for its value, makes sure everything is alright with return types and whatnot,
    /// then returns the raw, yielded data.
    pub async fn receive_from_channels(&mut self) -> R<Vec<ModuleYield>> {
        #[cfg(debug_assertions)]
        if self.sender.is_some() {
            unreachable!("Runtime functions out of order! Backend initializer has not dropped its internal sender!");
        }

        const SECOND: Duration = Duration::from_secs(1);

        let timeout = self
            .config
            .start_timeout_seconds
            .unwrap_or(DEFAULT_START_TIMEOUT_SECONDS);

        // let mut last_recv = Mutex::new(tokio::time::Instant::now());
        let last_recv = Cell::new(tokio::time::Instant::now());
        let mut results = Vec::new();

        let listener_future = async {
            while let Some(yielded) = self.receiver.recv().await {
                // we have to refresh the counter or it may close us unexpectedly!
                last_recv.replace(tokio::time::Instant::now());

                // this runtime checking
                match self.expected_modules.remove(&yielded.id) {
                    Some(expected_type) => debug!("Received module with id {}: Expected module type {}, received module type {}", yielded.id, expected_type, yielded.module_type),
                    None => bail!("Module with id of {} has no module type!", yielded.id),
                }

                results.push(yielded);

                if self.expected_modules.is_empty() {
                    break;
                }
            }

            if !self.expected_modules.is_empty() {
                for (id, mod_type) in self.expected_modules.iter() {
                    warn!("Failed to yield module with id of {id}, module type of {mod_type}");
                }
                bail!("Some modules failed to yield!");
            }

            Ok(())
        };

        // This will only return if the timeout is hit.
        // If it is not hit, it will stop being polled and drop when the function is done.
        // Since it is the time since last message, the time elapsed must be queried on every loop.
        // I would like to use a simple integer, but that would mean absolute time since init start,
        // and some custom modules might take a long time.
        let timeout_future = async {
            loop {
                let last_recv_seconds = last_recv.get().elapsed().as_secs();

                if last_recv_seconds > timeout {
                    return Err::<(), _>(Report::msg("Reached timeout!"));
                }
                tokio::time::sleep(SECOND).await;
            }
        };

        // This returns when the first one returns
        select! {
            timeout = timeout_future => {
                timeout?;
            }
            listened = listener_future => {
                listened?;
            }
        }

        Ok(results)
    }
}
