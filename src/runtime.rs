use crate::modules::*;
use crate::prelude::*;
use tokio::runtime::Runtime;

struct BackendInitializer {
    runtime: Arc<Runtime>,
    receiver: mpsc::UnboundedReceiver<ModuleYield>,
    expected_modules: AHashMap<ModuleId, ModuleType>,
}
impl BackendInitializer {
    #[instrument(level = "trace", skip_all)]
    pub async fn new(runtime: Arc<Runtime>, config: ModulesKnown) -> R<Self> {
        let mut expected_modules = AHashMap::new();

        let mut module_id_creator = ModuleIdCreator::default();

        // Each module must send a listener to this channel when they are ready to push data.
        // I made it this way because rust doesn't have generator functions, and I needed a way for functions to yield values
        // when I need them, and for them to just run as single instances. A bunch of dbus proxy lifetime stuff is involved there too.
        // It would have been messy to make a Module::new() and Module::run() thing when the run method would have just errored out instantly.
        let (sender, receiver) = mpsc::unbounded_channel();
        let module_sender: Arc<mpsc::UnboundedSender<_>> = Arc::new(sender);

        macro_rules! init_module {
            ([$mod_name:expr] $mod_path:path, $input:expr) => {
                ::paste::paste! {
                    let [<$mod_name _yield_sender>] = Arc::clone(&module_sender);
                    let [<$mod_name _input>] = $input;
                    let [<$mod_name _id>] = module_id_creator.create();

                    expected_modules.insert([<$mod_name _id>], $mod_path::MODULE_TYPE);
                }
            };
        }

        Ok(Self {
            runtime,
            receiver,
            expected_modules,
        })
    }
    #[instrument(level = "trace", skip_all)]
    pub async fn listen(&mut self) -> R<()> {
        todo!();
    }
}

pub async fn run(runtime: Arc<Runtime>, config: ModulesKnown) -> R<()> {
    // TODO: Make these in macros
    // let system_conn = SystemConnection::new().await?;

    let my_sender = Arc::clone(&sender);

    // receive them all -- This stops when it has either accounted for all messages, or has waited for the timeout.

    let mut dynamic_modules = Vec::new();
    let mut static_modules = Vec::new();

    drop(sender);
    let mut timeout_count = 0u64;

    // timeout_count = 0;
    //             match mod_type {
    //                 ModuleType::OneShot(m) => static_modules.push(m),
    //                 ModuleType::Loop(c) => dynamic_modules.push(c),
    //             }
    while !expected_modules.is_empty() {
        // just get all the stuff that is waiting. I do it this way
        // because I don't want to mess with select! for critical code like this,
        // and most of this stuff should probably already be ready by now.
        match module_receiver.try_recv() {
            Ok((output_type, mod_type)) => {
                match output_type {
                    OutputType::Loop(l) => dynamic_modules.push(l),
                    OutputType::OneShot(m) => static_modules.push(m),
                }

                expected_module_types.remove(&mod_type);
                timeout_count = 0;
            }
            Err(mpsc::error::TryRecvError::Disconnected) => break,
            Err(mpsc::error::TryRecvError::Empty) => {
                timeout_count += 1;
                if timeout_count > config.start_timeout_seconds {
                    warn!("Timeout reached waiting for modules");
                    break;
                }
                tokio::time::sleep(Duration::from_secs(config.start_timeout_seconds)).await;
            }
        }
    }

    // I don't want to just bail! here because I don't want people's bar to break
    // if, say, they get an update with breaking changes.
    if !expected_module_types.is_empty() {
        error!(
            "Missing modules: {}",
            expected_module_types
                .iter()
                .map(AsRef::as_ref)
                .collect::<Box<[&str]>>()
                .join(", ")
        );
    }

    tokio::task::block_in_place(|| loop {});
    Ok(())
}
