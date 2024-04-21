use std::ops::Deref;

use crate::imports::*;

#[derive(Debug)]
pub struct Interface {
    socket_path: PathBuf,
    state: InterfaceState,
    /// A Sender to send messages to the socket
    pub sender: Arc<mpsc::UnboundedSender<Message>>,
    my_receiver: mpsc::UnboundedReceiver<Message>,
    sub_sender: Arc<watch::Sender<Message>>,
    /// receiver to receive messages from the socket
    pub receiver: Arc<watch::Receiver<Message>>,
    socket: UnixListener,
}
impl Interface {
    /// Create a new [`Server`] to primarily write to the socket
    #[cfg_attr(feature = "tracing", ::tracing::instrument(level = "debug", skip_all))]
    pub async fn new() -> Result<Self, Error> {
        let socket_path = crate::get_socket_path()?;

        let state = if socket_path.exists() {
            InterfaceState::Client
        } else {
            InterfaceState::PotentialServer
        };

        // let stream = UnixStream::connect(&socket_path).await?;
        let socket = UnixListener::bind(&socket_path)?;

        let (s, my_receiver) = mpsc::unbounded_channel();
        let (ss, sr) = watch::channel(Message::default());

        Ok(Self {
            socket_path,
            state,
            sender: Arc::new(s),
            my_receiver,
            sub_sender: Arc::new(ss),
            receiver: Arc::new(sr),
            socket,
        })
    }
    /// Act as a server for the socket.
    ///
    /// There can be only one server accepting connections, this will return an error if there is already one.
    ///
    /// Consider spawning this on another async task, or joining this with another future that runs indefinitely.
    pub async fn server(&mut self) -> Result<(), Error> {
        self.state.try_server()?;
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
            let handle =
                tokio::spawn(async move { Self::read_socket_forever(owned_sender, stream).await });
            handles.push(handle);
        }

        while let Some(join) = handles.next().await {
            join??;
        }

        Err(Error::EarlyReturn)
    }
    async fn read_socket_forever(
        sender: Arc<watch::Sender<Message>>,
        stream: UnixStream,
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

                    sender.send_if_modified(|old| {
                        if *old != msg {
                            let _ = std::mem::replace(old, msg);
                            return true;
                        }

                        false
                    });

                    partial_line.clear();
                }
            }
        }
    }
    /// Send a message to the socket
    pub fn send_message(&self, message: Message) -> Result<(), Error> {
        self.sender.send(message)?;
        Ok(())
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
