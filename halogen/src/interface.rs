use futures_util::{stream::FuturesOrdered, TryStreamExt};

use crate::imports::*;

/// The max size of the chunks that I read from the socket
const BUFFER_SIZE: usize = 2048;

/// The interface for a socket. This should be a singleton.
#[derive(Debug)]
pub struct Interface {
    socket_path: Arc<PathBuf>,
    state: InterfaceState,
    sock_receiver: Arc<flume::Receiver<Message>>,
    sub_sender: Arc<flume::Sender<Message>>,
}
impl Interface {
    #[cfg(not(feature = "bin"))]
    #[inline]
    pub async fn new() -> Result<(Self, InterfaceStub), Error> {
        Self::new_inner(crate::get_socket_path()?).await
    }
    #[cfg(feature = "bin")]
    #[inline]
    pub async fn new(maybe_path: Option<&Path>) -> Result<(Self, InterfaceStub), Error> {
        let sock_path = match maybe_path {
            Some(p) => p.to_path_buf(),
            None => crate::get_socket_path()?,
        };
        Self::new_inner(sock_path).await
    }

    #[instrument(level = "debug")]
    async fn new_inner(socket_path: PathBuf) -> Result<(Self, InterfaceStub), Error> {
        let state = if socket_path.exists() {
            InterfaceState::Potential(InterfaceType::Client)
        } else {
            InterfaceState::Potential(InterfaceType::Server)
        };

        let socket_path = Arc::new(socket_path);

        let (s, sock_receiver) = flume::unbounded();
        let (sub_sender, sr) = flume::unbounded();

        let me = Self {
            socket_path: Arc::clone(&socket_path),
            state,
            sock_receiver: Arc::new(sock_receiver),
            sub_sender: Arc::new(sub_sender),
        };

        let stub = InterfaceStub {
            socket_path,
            sender: Arc::new(s),
            receiver: Arc::new(sr),
        };

        Ok((me, stub))
    }
    /// Act as a socket client, sending and receiving messages.
    ///
    /// There can be multiple clients, but they all get the same messages -- try not to make multiple in your crate!
    ///
    /// Consider spawning this on another async task, or joining this with another future that runs indefinitely.
    pub async fn client(&mut self) -> Result<(), Error> {
        self.state.try_as(InterfaceType::Client)?;

        if !self.path().exists() {
            return Err(Error::InvalidSocketPath(self.path().to_owned()));
        }

        let stream = UnixStream::connect(self.path()).await?;

        let owned_sender = Arc::clone(&self.sub_sender);
        let owned_receiver = Arc::clone(&self.sock_receiver);

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

        let socket = UnixListener::bind(self.path())?;
        let mut handles = FuturesUnordered::new();

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
            let owned_receiver = Arc::clone(&self.sock_receiver);
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
            if let Some(message_serialized) = format_message_for_sender(&message) {
                stream.writable().await?;
                stream.try_write(message_serialized.as_slice())?;
            }
        }
    }

    // #[instrument(level = "debug", skip_all)]
    async fn read_socket_forever(
        sender: Arc<flume::Sender<Message>>,
        stream: &UnixStream,
    ) -> Result<(), Error> {
        let mut line_buffer = SmallVec::<[u8; BUFFER_SIZE]>::new_const();

        loop {
            stream.readable().await?;
            let mut read_buffer = [0; BUFFER_SIZE];

            // It is okay to loop like this, that function joins multiple futures that should all be in order.
            while stream.try_read(&mut read_buffer)? > 0 {
                deserialize_bytes(&mut line_buffer, read_buffer, &sender).await?;
                // if something weird happens with my channel and it failed, that would be a shame
                stream.readable().await?;
            }
        }
    }
    /// Get the socket path
    #[inline]
    pub fn path<'p>(&'p self) -> &'p Path {
        &self.socket_path
    }
}
impl Drop for Interface {
    fn drop(&mut self) {
        match remove_socket(self.socket_path.as_path(), self.state) {
            Ok(_) => debug!(
                "Interface removed socket path: {}",
                self.socket_path.display()
            ),
            Err(e) => error!(
                "Interface failed to remove socket path: {e} at {}",
                self.socket_path.display()
            ),
        }
    }
}

/// Remove the socketfile. This is used internally by any server impl.
#[instrument(level = "debug", skip_all)]
fn remove_socket(socket: &Path, interface_state: InterfaceState) -> Result<(), Error> {
    if interface_state == InterfaceState::Current(InterfaceType::Server) {
        std::fs::remove_file(socket)?;
    } else {
        return Err(Error::InvalidState(interface_state));
    }

    Ok(())
}

/// The type of interface this is.
///
/// This is very similar to [`crate::SenderType`] but it fulfills a different purpose
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bin", derive(clap::ValueEnum))]
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
    /// Literally just here for the server drop
    socket_path: Arc<PathBuf>,

    pub sender: Arc<flume::Sender<Message>>,
    pub receiver: Arc<flume::Receiver<Message>>,
}
impl InterfaceStub {
    #[inline]
    pub fn clone_of_arc(&self) -> Self {
        Self {
            socket_path: Arc::clone(&self.socket_path),
            sender: Arc::clone(&self.sender),
            receiver: Arc::clone(&self.receiver),
        }
    }
    /// Send a messsage to the internal sender.
    #[inline]
    pub fn send(&self, message: Message) -> Result<(), flume::SendError<Message>> {
        self.sender.send(message)
    }
    /// Try to drop (remove) the socket path
    ///
    /// This only works if the current [`Interface`] is a server.
    #[inline]
    pub fn drop_path(&self) -> Result<(), Error> {
        remove_socket(
            self.socket_path.as_path(),
            InterfaceState::Current(InterfaceType::Server),
        )?;
        Ok(())
    }
}

/// If you use this in more than one spot in the code I will shank you
///
/// Format the message properly to be sent over the wire
///
/// Trust me, you don't wanna know any more than this.
#[instrument(level = "trace", skip_all)]
fn format_message_for_sender(message: &Message) -> Option<Vec<u8>> {
    let mut buffer = match json::to_vec(message) {
        Ok(b) => b,
        Err(e) => {
            warn!("Failed to serialize message: {e}");
            return None;
        }
    };

    // insert the API version as the first byte. This is so that I don't have to retry parsing 5 times for 5 different versions.
    buffer.insert(0, crate::LATEST_API_VERSION);
    // null-terminated strings in rust?????? (I didn't want to do this but I had no choice)
    buffer.push(0);
    Some(buffer)
}

/// If you use this in more than one spot in the code I will shank you
///
/// This takes the sender because it is an internal API and I don't want to allocate a buffer just to pass data
/// into something that was just going to pass the data somewhere else.
///
/// This takes a mutable Vec that is supposed to be a temp buffer, storing part of the line. Don't touch it, it is there so I
/// don't allocate a shit ton of RAM.
///
/// chillax, it's an internal API
#[instrument(level = "trace", skip_all)]
async fn deserialize_bytes(
    partial_message: &mut SmallVec<[u8; BUFFER_SIZE]>,
    bytes: [u8; BUFFER_SIZE],
    sender: &flume::Sender<Message>,
) -> Result<(), Error> {
    // TODO: Test if this actually maintains order in this case
    let mut sends = FuturesOrdered::new();
    for byte in bytes {
        if byte != 0 {
            partial_message.push(byte);
            continue;
            // ends this iteration early so I don't have to indent this all the way to Saturn
        }
        let api_version = partial_message.remove(0);
        if api_version > crate::LATEST_API_VERSION {
            return Err(Error::InvalidApiVersion(api_version));
        }

        // safety: I am clearing this Vec after calling this
        let message = match json::from_slice(partial_message.as_mut_slice()) {
            Ok(m) => m,
            Err(e) => {
                warn!("Halogen server json decoding error: {e}");
                // it isn't the end of the world, it's just one message
                continue;
            }
        };
        partial_message.clear();

        sends.push_back(sender.send_async(message))
    }

    loop {
        if sends.try_next().await?.is_none() {
            break;
        }
    }
    Ok(())
}

/// Messages sent to the interface for use internally
pub enum InterfaceMessage {
    DropSocket,
    // more TBD
}
