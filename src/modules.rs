pub mod network;
pub mod time;
use tokio::runtime::Runtime;
use tracing::Instrument;

use crate::prelude::*;

config_struct! {
    [Modules]
    start_timeout_seconds: u16 = 5,
}

pub async fn run(runtime: Arc<Runtime>, config: ModulesKnown) -> R<()> {
    // TODO: Make these in macros
    let time_config = time::TimeKnown::default();
    let (mut time_module, mut time_channel) = time::Time::new((), time_config).await?;

    let mut my_receiver = time_channel.get_receiver().expect(const_format::formatcp!(
        "Failed to acquire {} receiver",
        "time"
    ));
    // TODO: Maybe do all string formatting on a blocking thread or two
    runtime.spawn(async move {
        while let Some(m) = my_receiver.recv().await {
            info!("Time: {m}");
        }
    });

    runtime.spawn(async move {
        time_module.run().await?;
        Ok::<_, Report>(())
    });

    let dura = Duration::from_secs(5);

    runtime.spawn(async move {
        loop {
            tokio::time::sleep(dura).await;
            let span = tracing::info_span!("Time channel send");
            time_channel.send(Event::Click).instrument(span).await;
        }
    });

    let system_conn = SystemConnection::new().await?;

    // Each module must send a listener to this channel when they are ready to push data.
    let (sender, module_receiver) = mpsc::unbounded_channel();
    let sender = Arc::new(sender);

    let my_conn = system_conn.clone();
    let my_sender = Arc::clone(&sender);
    let my_rt = Arc::clone(&runtime);

    runtime.spawn(async move {
        let config = network::NetKnown::default(); // init(config, my_conn, sender).await?;
        network::Network::run(my_rt, (config, my_conn), my_sender).await?;
        Ok::<_, Report>(())
    });

    // receive them all -- This stops when it has either accounted for all messages, or has waited for the timeout.

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
    /// The function that runs this module. Consider this function blocking.
    ///
    /// Important: If it is a oneshot with no events, please specify! If it has to receive events, make it a loop.
    ///
    /// If it was expected to return, it will return `Ok(true)`. A bool value of `false` indicates it was supposed to run forever.
    async fn run(
        runtime: Arc<Runtime>,
        input: Self::Input,
        yield_sender: Arc<mpsc::UnboundedSender<ModuleType>>,
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

/// A two-way mpsc channel.
///
/// TODO: Document more
pub struct BiChannel<T, F> {
    pub context: String,
    pub sender: mpsc::Sender<T>,
    /// This is an Option so that modules can acquire it in `async move` closures
    pub receiver: Option<mpsc::Receiver<F>>,
}
impl<T, F> BiChannel<T, F> {
    /// Create a new two-way mpsc channel. The buffer is the number of messages it holds before applying backpressure,
    /// and the context is the string that it logs just in case of any errors during the course of its operation.
    pub fn new<S: Into<String>>(
        buffer: usize,
        first_context: Option<S>,
        second_context: Option<S>,
    ) -> (BiChannel<T, F>, BiChannel<F, T>) {
        let (sender1, receiver1) = mpsc::channel(buffer);
        let (sender2, receiver2) = mpsc::channel(buffer);

        (
            BiChannel {
                context: match first_context {
                    Some(s) => s.into(),
                    None => "None".to_owned(),
                },
                sender: sender1,
                receiver: Some(receiver2),
            },
            BiChannel {
                context: match second_context {
                    Some(s) => s.into(),
                    None => "None".to_owned(),
                },
                sender: sender2,
                receiver: Some(receiver1),
            },
        )
    }
    /// Try to get this channel's receiver. Receivers are Options so that you can use them in `async move` infinite loops.
    #[inline]
    pub fn get_receiver(&mut self) -> Option<mpsc::Receiver<F>> {
        self.receiver.take()
    }
    /// Try to send a message through the channel. If it succeeds, this returns true.
    /// If it fails, it logs an error and returns false.
    pub async fn send(&self, item: T) -> bool {
        match self.sender.send(item).await {
            Ok(()) => true,
            Err(e) => {
                error!(
                    "Failed to send message to BiChannel({}): {e}",
                    &self.context
                );
                false
            }
        }
    }
}

/// An enum to assist modules that have multiple formatting states
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatState {
    #[default]
    Normal,
    Alternate,
}
impl FormatState {
    /// Switch the current state to the next available.
    pub fn next(&mut self) {
        let next = match self {
            Self::Normal => Self::Alternate,
            Self::Alternate => Self::Normal,
        };
        *self = next;
    }
}

/// Content that can be printed by the frontend.
///
/// To use this, impl `Into<DisplayOutput>` for `T`.
///
/// TODO: Finalize stuff required. This is just a string temporarily.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_more::Display, derive_more::From,
)]
pub struct DisplayOutput(String);

/// The type of module that this is. This determines a lot about how it is run.
pub enum ModuleType {
    /// The module returns a constant through its channel on start, and is not run.
    OneShot(DisplayOutput),
    /// The module runs in a loop, pushing changes through its channel. The run function should never exit.
    Loop(BiChannel<Event, DisplayOutput>),
}

/// A [`zbus::Connection`] that contains a connection to the system bus
#[derive(Debug, Clone, derive_more::AsRef)]
pub struct SystemConnection(pub zbus::Connection);
impl SystemConnection {
    pub async fn new() -> zbus::Result<Self> {
        let conn = zbus::Connection::system().await?;
        Ok(Self(conn))
    }
}
/// A [`zbus::Connection`] that contains a connection to the session bus
#[derive(Debug, Clone, derive_more::AsRef)]
pub struct SessionConnection(pub zbus::Connection);
impl SessionConnection {
    pub async fn new() -> zbus::Result<Self> {
        let conn = zbus::Connection::session().await?;
        Ok(Self(conn))
    }
}
