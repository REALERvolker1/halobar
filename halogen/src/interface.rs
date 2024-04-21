use std::intrinsics::unreachable;

use futures_util::TryFutureExt;
use tokio::task::JoinHandle;

use crate::imports::*;

#[derive(Debug)]
pub struct Interface {
    socket_path: PathBuf,
    state: InterfaceState,
    my_receiver: Arc<flume::Receiver<Message>>,
    sub_sender: Arc<flume::Sender<Message>>,
    socket: UnixListener,
}
impl Interface {
    /// Create a new [`Server`] to primarily write to the socket
    #[cfg_attr(feature = "tracing", ::tracing::instrument(level = "debug", skip_all))]
    pub async fn new() -> Result<(Self, InterfaceStub), Error> {
        let socket_path = crate::get_socket_path()?;

        let state = if socket_path.exists() {
            InterfaceState::Client
        } else {
            InterfaceState::PotentialServer
        };

        // let stream = UnixStream::connect(&socket_path).await?;
        let socket = UnixListener::bind(&socket_path)?;

        let (s, my_receiver) = flume::unbounded();
        let (sub_sender, sr) = flume::unbounded();

        let me = Self {
            socket_path,
            state,
            my_receiver: Arc::new(my_receiver),
            sub_sender: Arc::new(sub_sender),
            socket,
        };

        let stub = InterfaceStub {
            sender: Arc::new(s),
            receiver: Arc::new(sr),
        };

        Ok((me, stub))
    }
    /// Act as a server for the socket.
    ///
    /// There can be only one server accepting connections, this will return an error if there is already one.
    ///
    /// Consider spawning this on another async task, or joining this with another future that runs indefinitely.
    pub async fn server(&mut self) -> Result<(), Error> {
        self.state.try_server()?;

        let my_serve = async {
            let mut handles = futures_util::stream::FuturesUnordered::new();

            loop {
                let (stream, address) = match self.socket.accept().await {
                    Ok(s) => s,
                    Err(e) => {
                        #[cfg(feature = "tracing")]
                        tracing::warn!("Halogen server could not accept connection: {e}");
                        break;
                    }
                };

                #[cfg(feature = "tracing")]
                tracing::debug!(
                    "Halogen server received connection from address: {:?}",
                    address.as_pathname()
                );
                let owned_sender = Arc::clone(&self.sub_sender);
                let owned_receiver = Arc::clone(&self.my_receiver);
                let handle = tokio::spawn(async move {
                    let res = tokio::try_join!(
                        Self::read_socket_forever(owned_sender, &stream),
                        Self::write_socket_forever(owned_receiver, &stream)
                    );

                    match res {
                        Ok(((), ())) => unreachable!(),
                        Err(e) => tracing::error!("{e}"),
                    }
                });
                handles.push(handle);
            }

            while let Some(join) = handles.next().await {
                join?;
            }

            Err::<(), _>(Error::EarlyReturn)
        };

        // let my_recv = async {
        //     let mut recv = self.my_receiver.into_stream();

        //     Err::<(), _>(Error::EarlyReturn)
        // };

        my_serve.await?;

        Err(Error::EarlyReturn)
    }
    async fn write_socket_forever(
        receiver: Arc<flume::Receiver<Message>>,
        stream: &UnixStream,
    ) -> Result<(), Error> {
        loop {
            let message = receiver.recv_async().await?;
            let message_serialized = match message.into_json() {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!("Failed to serialize message: {e}");
                    continue;
                }
            };
            stream.writable().await?;
            stream.try_write(message_serialized.as_bytes())?;
        }
    }
    // /// Receive messages from [`InterfaceStub`]s and send them to the channel.
    // pub async fn receive_messages(&self)
    async fn read_socket_forever(
        sender: Arc<flume::Sender<Message>>,
        stream: &UnixStream,
    ) -> Result<(), Error> {
        let mut partial_line = String::new();

        loop {
            stream.readable().await?;

            loop {
                let mut buffer = [0; 2048];
                let read = stream.try_read(&mut buffer)?;

                if read == 0 {
                    break;
                }

                let decoded = match std::str::from_utf8(&buffer) {
                    Ok(s) => s,
                    Err(e) => {
                        #[cfg(feature = "tracing")]
                        tracing::warn!("Halogen server decoding error: {e}");
                        continue;
                    }
                };

                for char in decoded.chars() {
                    if char != '\n' {
                        partial_line.push(char);
                        continue;
                    }

                    // TODO: Test if futures ordered is good for this
                    let msg = match Message::try_from_raw(&partial_line) {
                        Ok(m) => m,
                        Err(e) => {
                            #[cfg(feature = "tracing")]
                            tracing::warn!("Halogen server decoding error: {e}");
                            // it isn't the end of the world, it's just one message
                            return Ok(());
                        }
                    };

                    // A server sent the message, ignore
                    if msg.sender_type() == crate::SenderType::Server {
                        continue;
                    }

                    sender.send(msg)?;

                    partial_line.clear();
                }
            }
        }
    }
}
impl Drop for Interface {
    fn drop(&mut self) {
        match self.state {
            InterfaceState::Client => {}
            InterfaceState::PotentialClient | InterfaceState::PotentialServer => {}
            InterfaceState::Server => {
                if self.socket_path.is_file() {
                    if let Err(e) = std::fs::remove_file(&self.socket_path) {
                        tracing::error!(
                            "Failed to remove socket path: {e} at {}",
                            self.socket_path.display()
                        );
                    }
                } else {
                    tracing::debug!("Removing socket path: {}", self.socket_path.display());
                }
            }
        }
    }
}

/// Determines what the interface can be and do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceState {
    Client,
    Server,
    /// The socket path doesn't exist yet
    PotentialServer,
    PotentialClient,
}
impl InterfaceState {
    /// Try to set this to a server. Used internally.
    #[inline]
    fn try_server(&mut self) -> Result<(), Error> {
        if *self == InterfaceState::PotentialServer {
            *self = Self::Server;
        }
        Err(Error::InvalidState(*self))
    }
}

/// A stub, meant to facilitate listening to an interface.
///
/// Uses `Arc` internally so it is cheap to clone.
#[derive(Debug, Clone)]
pub struct InterfaceStub {
    pub sender: Arc<flume::Sender<Message>>,
    pub receiver: Arc<flume::Receiver<Message>>,
}
impl InterfaceStub {
    /// Send a messsage to the internal sender.
    #[inline]
    pub fn send(&self, message: Message) -> Result<(), flume::SendError<Message>> {
        self.sender.send(message)
    }
}
