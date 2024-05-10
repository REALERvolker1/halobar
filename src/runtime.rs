use crate::modules::*;
use crate::prelude::*;
use tokio::runtime::Runtime;

#[inline]
pub async fn run(runtime: Arc<Runtime>, config: ModulesKnown) -> R<()> {
    let initializer = BackendInitializer::new(runtime, config).await?;

    initializer.run().await
}

config_struct! {
    [Modules]
    // @conf network: network => Net,
    @conf time: time => Time,
    start_timeout_seconds: u64 = 5,
}

struct BackendInitializer {
    runtime: Arc<Runtime>,
    module_id_creator: ModuleIdCreator,
    receiver: mpsc::UnboundedReceiver<ModuleYield>,
    sender: Arc<mpsc::UnboundedSender<ModuleYield>>,
    expected_modules: AHashMap<ModuleId, ModuleType>,
    config: ModulesKnown,
}
impl BackendInitializer {
    /// Internal backend initializer creation function
    pub async fn new(runtime: Arc<Runtime>, config: ModulesKnown) -> R<Self> {
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
        let mut handles = FuturesUnordered::new();

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
            input: self.config.time,
        }

        // Get the channels, and start listeners from this function!
        wait_for_return(handles).await;
        info!("Finished backend execution!");
        Ok(())
    }
    pub async fn watch_channels(&mut self) -> R<()> {
        todo!();
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
