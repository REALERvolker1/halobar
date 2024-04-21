use std::path::Path;

use crate::imports::*;

/// The interface for a socket. This is the primary singleton for the crate.
#[derive(Debug)]
pub struct Interface {
    socket_path: PathBuf,
    state: InterfaceState,
    my_receiver: Arc<flume::Receiver<Message>>,
    sub_sender: Arc<flume::Sender<Message>>,
}
impl Interface {
    /// Create a new [`Server`] to primarily write to the socket
    #[instrument(level = "debug", skip_all)]
    pub fn new() -> Result<(Self, InterfaceStub), Error> {
        let socket_path = crate::get_socket_path()?;

        let state = if socket_path.exists() {
            InterfaceState::Potential(InterfaceType::Client)
        } else {
            InterfaceState::Potential(InterfaceType::Server)
        };

        // let stream = UnixStream::connect(&socket_path).await?;
        // let socket = UnixListener::bind(&socket_path)?;

        let (s, my_receiver) = flume::unbounded();
        let (sub_sender, sr) = flume::unbounded();

        let me = Self {
            socket_path,
            state,
            my_receiver: Arc::new(my_receiver),
            sub_sender: Arc::new(sub_sender),
        };

        let stub = InterfaceStub {
            sender: Arc::new(s),
            receiver: Arc::new(sr),
        };

        Ok((me, stub))
    }
    /// Act as a socket interface, sending and receiving messages -- like a client.
    ///
    /// There can be multiple clients, but they all get the same messages -- try not to make multiple in your crate!
    ///
    /// Consider spawning this on another async task, or joining this with another future that runs indefinitely.
    pub async fn interface(&mut self) -> Result<(), Error> {
        self.state.try_as(InterfaceType::Client)?;

        let stream = UnixStream::connect(&self.socket_path).await?;

        let owned_sender = Arc::clone(&self.sub_sender);
        let owned_receiver = Arc::clone(&self.my_receiver);

        tokio::try_join!(
            Self::read_socket_forever(owned_sender, &stream),
            Self::write_socket_forever(owned_receiver, &stream)
        )?;

        Err(Error::EarlyReturn)
    }
    /// Act as a server for the socket.
    ///
    /// There can be only one server accepting connections, this will return an error if there is already one.
    ///
    /// Consider spawning this on another async task, or joining this with another future that runs indefinitely.
    pub async fn server(&mut self) -> Result<(), Error> {
        self.state.try_as(InterfaceType::Server)?;

        let socket = UnixListener::bind(&self.socket_path)?;
        let mut handles = futures_util::stream::FuturesUnordered::new();

        loop {
            let (stream, address) = match socket.accept().await {
                Ok(s) => s,
                Err(e) => {
                    warn!("Halogen server could not accept connection: {e}, stopping listener");
                    break;
                }
            };
            debug!(
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
                    Err(e) => error!("{e}"),
                }
            });
            handles.push(handle);
        }

        while let Some(join) = handles.next().await {
            join?;
        }

        Err(Error::EarlyReturn)
    }
    #[instrument(level = "debug", skip_all)]
    async fn write_socket_forever(
        receiver: Arc<flume::Receiver<Message>>,
        stream: &UnixStream,
    ) -> Result<(), Error> {
        loop {
            let message = receiver.recv_async().await?;
            let message_serialized = match message.into_json() {
                Ok(m) => m,
                Err(e) => {
                    warn!("Failed to serialize message: {e}");
                    continue;
                }
            };
            stream.writable().await?;
            stream.try_write(message_serialized.as_bytes())?;
        }
    }
    // #[instrument(level = "debug", skip_all)]
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
                        warn!("Halogen server decoding error: {e}");
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
                            warn!("Halogen server decoding error: {e}");
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
    /// remove socket when this is done
    pub fn drop_file(&mut self) {
        match self.state {
            InterfaceState::Client => {}
            InterfaceState::PotentialClient | InterfaceState::PotentialServer => {}
            InterfaceState::Server => {
                if self.socket_path.is_file() {
                    if let Err(e) = std::fs::remove_file(&self.socket_path) {
                        error!(
                            "Failed to remove socket path: {e} at {}",
                            self.socket_path.display()
                        );
                    }
                } else {
                    debug!("Removing socket path: {}", self.socket_path.display());
                }
            }
        }
    }
}
impl Drop for Interface {
    fn drop(&mut self) {
        self.drop_file()
    }
}

/// The type of interface this is
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceType {
    Client,
    Server,
}
/// Determines what the interface can be and do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceState {
    Potential(InterfaceType),
    Current(InterfaceType),
}
impl InterfaceState {
    /// Get the interface type. Both have one, so this will not fail.
    pub fn unwrap_type(&self) -> InterfaceType {
        *match self {
            Self::Current(t) => t,
            Self::Potential(t) => t,
        }
    }
    #[inline]
    fn try_as(&mut self, try_type: InterfaceType) -> Result<(), Error> {
        if let Self::Potential(i) = *self {
            if i == try_type {
                *self = Self::Current(i);
                return Ok(());
            }
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
