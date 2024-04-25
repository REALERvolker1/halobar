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
pub trait BackendModule: Sized {
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
    pub fn new(
        buffer: usize,
        first_context: Option<String>,
        second_context: Option<String>,
    ) -> (BiChannel<T, F>, BiChannel<F, T>) {
        let (sender1, receiver1) = mpsc::channel(buffer);
        let (sender2, receiver2) = mpsc::channel(buffer);

        (
            BiChannel {
                context: first_context.unwrap_or_else(|| "None".to_owned()),
                sender: sender1,
                receiver: Some(receiver2),
            },
            BiChannel {
                context: second_context.unwrap_or_else(|| "None".to_owned()),
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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FormatState {
    #[default]
    Normal,
    Alternate,
}
impl FormatState {
    /// Switch the current state to the next available
    pub fn next(&mut self) {
        let next = match self {
            Self::Normal => Self::Alternate,
            Self::Alternate => Self::Normal,
        };
        *self = next;
    }
}
