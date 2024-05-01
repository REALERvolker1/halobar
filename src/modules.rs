pub mod network;
pub mod time;

use crate::prelude::*;

pub async fn run(runtime: &tokio::runtime::Runtime) -> R<()> {
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

    let dbus_conn = zbus::Connection::system().await?;

    let my_conn = dbus_conn.clone();

    let (sender, network_receiver) = oneshot::channel();
    runtime.spawn(async move {
        let config = network::NetKnown::default();
        network::Network::init(config, my_conn, sender).await?;
        Ok::<_, Report>(())
    });

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
use tokio::runtime::Runtime;
use tracing::Instrument;

pub trait BackendModule2: Sized + Send {
    /// The type of config that the module requires for user customization.
    type Config;
    /// The type of input that the module requires to create a new instance.
    type Input;
    /// The type of error that the module can return
    type Error;
    /// The function that runs this module.
    async fn init<D: Into<DisplayOutput>>(
        runtime: Runtime,
        config: Self::Config,
        input: Self::Input,
        sender: oneshot::Sender<BiChannel<Event, D>>,
    );
}

/// A module that can be used in the backend to provide data.
pub trait BackendModule: Sized + Send {
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
    /// Get this module's [`ModuleType`]. Ideally should be inlined.
    fn module_type() -> ModuleType;
}

/// A two-way mpsc channel.
///
/// TODO: Document more
pub struct BiChannel<T, F> {
    pub context: String,
    pub sender: Sender<T>,
    /// This is an Option so that modules can acquire it in `async move` closures
    pub receiver: Option<Receiver<F>>,
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
    pub fn get_receiver(&mut self) -> Option<Receiver<F>> {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
pub enum ModuleType {
    /// The module returns a constant string through its channel on start, and is not run.
    Constant,
    /// The module is run once.
    Oneshot,
    /// The module runs in a loop, pushing changes through its channel. The run function should never exit.
    Loop,
}

/// A [`zbus::Connection`] that contains a connection to the system bus
pub struct SystemConnection(pub zbus::Connection);
impl SystemConnection {
    pub async fn new() -> zbus::Result<Self> {
        let conn = zbus::Connection::system().await?;
        Ok(Self(conn))
    }
}
/// A [`zbus::Connection`] that contains a connection to the session bus
pub struct SessionConnection(pub zbus::Connection);
impl SessionConnection {
    pub async fn new() -> zbus::Result<Self> {
        let conn = zbus::Connection::session().await?;
        Ok(Self(conn))
    }
}
