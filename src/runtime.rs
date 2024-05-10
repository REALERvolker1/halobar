use crate::modules::{self, BackendModule, ModuleType, ModuleYield};
use crate::prelude::*;
use tokio::runtime::Runtime;

#[inline]
pub async fn run(runtime: Arc<Runtime>, config: ModuleConfig) -> R<()> {
    let initializer = BackendInitializer::new(runtime, config).await?;

    initializer.run().await
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
    module_id_creator: ModuleIdCreator,
    receiver: mpsc::UnboundedReceiver<ModuleYield>,
    sender: Arc<mpsc::UnboundedSender<ModuleYield>>,
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
            module_id_creator: ModuleIdCreator::default(),
            receiver,
            sender: Arc::new(sender),
            expected_modules: AHashMap::new(),
            config,
        })
    }
    #[instrument(level = "trace", skip_all)]
    pub async fn run(mut self) -> R<()> {
        let handles = FuturesUnordered::new();

        macro_rules! init_module {
            ($( [$mod_name:expr] module: $mod_path:ty, input: $input:expr ),+$(,)?) => {$({
                let yield_sender = Arc::clone(&self.sender);
                let input = $input;
                let id = self.module_id_creator.create();

                trace!("Initializing module {}:{}", $mod_name, id);
                self.expected_modules.insert(id, <$mod_path>::MODULE_TYPE);

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

        // Get the channels, and start listeners from this function!
        let modules = self.receive_from_channels().await?;

        wait_for_return(handles).await;
        info!("Finished backend execution!");
        Ok(())
    }
    /// The third component of initialization.
    /// This waits for each module to return a listener for its value, makes sure everything is alright with return types and whatnot,
    /// then returns the raw, yielded data.
    pub async fn receive_from_channels(&mut self) -> R<Vec<ModuleYield>> {
        const SECOND: Duration = Duration::from_secs(1);

        let timeout = self
            .config
            .start_timeout_seconds
            .unwrap_or(DEFAULT_START_TIMEOUT_SECONDS);

        // let mut last_recv = Mutex::new(tokio::time::Instant::now());
        let last_recv = Cell::new(tokio::time::Instant::now());

        let listener_future = async {
            let mut results = Vec::new();

            while let Some(yielded) = self.receiver.recv().await {
                // we have to refresh the counter or it may close us unexpectedly!
                last_recv.replace(tokio::time::Instant::now());

                // this runtime checking
                match self.expected_modules.remove(&yielded.id) {
                    Some(expected_type) => debug!("Received module with id {}: Expected module type {}, received module type {}", yielded.id, expected_type, yielded.module_type),
                    None => bail!("Module with id of {} has no module type!", yielded.id),
                }

                results.push(yielded);
            }

            if !self.expected_modules.is_empty() {
                for (id, mod_type) in self.expected_modules.iter() {
                    warn!("Failed to yield module with id of {id}, module type of {mod_type}");
                }
                bail!("Some modules failed to yield!");
            }

            Ok(results)
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
                    bail!("Reached timeout!");
                }
                tokio::time::sleep(SECOND).await;
            }
        };

        let (results, _): (_, ()) = tokio::try_join!(listener_future, timeout_future)?;

        Ok(results)
    }
}

async fn wait_for_return(
    mut handles: FuturesUnordered<tokio::task::JoinHandle<(&'static str, Result<bool, Report>)>>,
) {
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
}
