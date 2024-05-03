pub mod network;
pub mod time;
use tokio::runtime::Runtime;
use tracing::Instrument;

use crate::prelude::*;

config_struct! {
    [Modules]
    @conf network: network => Net,
    start_timeout_seconds: u64 = 5,
}

pub async fn run(runtime: Arc<Runtime>, config: ModulesKnown) -> R<()> {
    // TODO: Make these in macros
    let system_conn = SystemConnection::new().await?;

    let mut expected_module_types = AHashSet::new();

    // Each module must send a listener to this channel when they are ready to push data.
    let (sender, mut module_receiver) = mpsc::unbounded_channel();
    let sender = Arc::new(sender);

    let my_conn = system_conn.clone();
    let my_sender = Arc::clone(&sender);
    let my_rt = Arc::clone(&runtime);

    if expected_module_types.contains(&network::Network::MODULE_TYPE) {
        bail!("Duplicate module found: {}!", network::Network::MODULE_TYPE);
    }
    expected_module_types.insert(network::Network::MODULE_TYPE);

    runtime.spawn(async move {
        let config = config.network;
        network::Network::run(my_rt, (config, my_conn), my_sender).await?;
        Ok::<_, Report>(())
    });

    // receive them all -- This stops when it has either accounted for all messages, or has waited for the timeout.
    // I made it this way because rust doesn't have generator functions, and I needed a way for functions to yield values
    // when I need them, and for them to just run as single instances. A bunch of dbus proxy lifetime stuff is involved there too.
    // It would have been messy to make a Module::new() and Module::run() thing when the run method would have just errored out instantly.

    let mut dynamic_modules = Vec::new();
    let mut static_modules = Vec::new();

    drop(sender);
    let mut timeout_count = 0u64;

    // timeout_count = 0;
    //             match mod_type {
    //                 ModuleType::OneShot(m) => static_modules.push(m),
    //                 ModuleType::Loop(c) => dynamic_modules.push(c),
    //             }
    while !expected_module_types.is_empty() {
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

/// A helper to make dbus proxy modules
#[macro_export]
macro_rules! proxy {
    ($conn:expr, $proxy:ty) => {
        <$proxy>::builder($conn)
            .cache_properties(::zbus::proxy::CacheProperties::No)
            .build()
    };
}
pub use proxy;

/// A module that can be used in the backend to provide data.
pub trait BackendModule: Sized + Send {
    /// The type of input that the module requires to create a new instance,
    /// including any type of config that the module requires for user customization.
    type Input;
    /// The type of error that the module can return
    type Error: Into<Report>;
    /// Get the requirements for this module to run. This is used to make sure we only initialize what we need.
    const MODULE_REQUIREMENTS: &'static [ModuleRequirementDiscriminants];
    /// The type of module this is. It is used to identify where the data should go.
    const MODULE_TYPE: ModuleType;
    /// The function that runs this module. Consider this function blocking.
    ///
    /// Important: If it is a oneshot with no events, please specify! If it has to receive events, make it a loop.
    ///
    /// If it was expected to return, it will return `Ok(true)`. A bool value of `false` indicates it was supposed to run forever.
    async fn run(
        runtime: Arc<Runtime>,
        input: Self::Input,
        yield_sender: Arc<mpsc::UnboundedSender<(OutputType, ModuleType)>>,
    ) -> Result<bool, Self::Error>;
}

/// TODO: Remove
pub trait BackendModule1: Sized + Send {
    /// The type of config that the module requires for user customization.
    type Config;
    /// The type of input that the module requires to create a new instance.
    type Input;
    /// The type of error that the module can return
    type Error;
    /// Create a new instance of this module.
    async fn new(
        input: Self::Input,
        config: Self::Config,
    ) -> Result<(Self, BiChannel<Event, String>), Self::Error>;
    /// Run this module. Whether this function loops forever, runs once, or is not run at all depends entirely on its module type.
    async fn run(&mut self) -> Result<(), Self::Error>;
    /// Listen for events with this module. Whether this function runs at all depends entirely on its module type.
    async fn receive_event(&self, event: Event) -> Result<(), Self::Error>;
    // /// Get this module's [`ModuleType`]. Ideally should be inlined.
    // fn module_type() -> ModuleType;
}

/// A specific requirement that the module needs to work properly
#[derive(Debug, strum_macros::EnumDiscriminants, strum_macros::EnumTryAs)]
#[strum_discriminants(derive(Serialize, Deserialize, strum_macros::Display))]
pub enum ModuleRequirement {
    SystemDbus(SystemConnection),
    SessionDbus(SessionConnection),
}
impl ModuleRequirement {
    /// Try to fulfill this
    #[inline]
    pub async fn fulfill_system_dbus(&self) -> zbus::Result<SystemConnection> {
        SystemConnection::new().await
    }
}

/// The type of module that this is. This determines a lot about how it is run.
#[derive(strum_macros::EnumDiscriminants)]
pub enum OutputType {
    /// The module returns a constant through its channel on start, and is not run.
    OneShot(DisplayOutput),
    /// The module runs in a loop, pushing changes through its channel. The run function should never exit.
    Loop(BiChannel<Event, DisplayOutput>),
}
