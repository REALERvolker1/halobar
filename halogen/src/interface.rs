use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use futures_util::StreamExt;

use crate::imports::*;

pub struct Server {
    /// A Sender to send messages to the socket
    pub sender: Arc<mpsc::UnboundedSender<Message>>,
    my_receiver: mpsc::UnboundedReceiver<Message>,
    sub_sender: Arc<watch::Sender<Message>>,
    pub receiver: Arc<watch::Receiver<Message>>,
    socket: UnixListener,
}
impl Server {
    /// Create a new [`Server`] to primarily write to the socket
    #[cfg_attr(feature = "tracing", ::tracing::instrument(level = "debug", skip_all))]
    pub async fn new() -> Result<Self, Error> {
        let socket_path = crate::get_socket_path()?;
        // let stream = UnixStream::connect(&socket_path).await?;
        let socket = UnixListener::bind(socket_path)?;

        let (s, my_receiver) = mpsc::unbounded_channel();
        let (ss, sr) = watch::channel(Message::default());

        Ok(Self {
            sender: Arc::new(s),
            my_receiver,
            sub_sender: Arc::new(ss),
            receiver: Arc::new(sr),
            socket,
        })
    }
    /// Read/write to the stream indefinitely. This should not return.
    ///
    /// Consider spawning this on another async task, or joining this with another future that runs indefinitely.
    pub async fn await_connections(&self) -> Result<(), Error> {
        loop {
            let (stream, address) = self.socket.accept().await?;
            #[cfg(feature = "tracing")]
            tracing::debug!(
                "Halogen server received connection from address: {:?}",
                address.as_pathname()
            );
            let owned_sender = Arc::clone(&self.sub_sender);
            tokio::spawn(async move { Self::read_socket_forever(owned_sender, stream).await });
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
