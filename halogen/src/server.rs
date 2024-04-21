use std::collections::VecDeque;

use crate::imports::*;

pub struct Server {
    /// A Sender to send messages to the socket
    pub sender: Arc<mpsc::UnboundedSender<Message>>,
    my_receiver: mpsc::UnboundedReceiver<Message>,
    sub_sender: watch::Sender<Message>,
    pub receiver: Arc<watch::Receiver<Message>>,
    socket: UnixListener,
    buffer: Vec<u8>,
}
impl Server {
    /// Create a new [`Server`] to primarily write to the socket
    #[cfg_attr(feature = "tracing", ::tracing::instrument(level = "debug", skip_all))]
    pub async fn new() -> Result<Self, Error> {
        let socket_path = crate::get_socket_path()?;
        // let stream = UnixStream::connect(&socket_path).await?;
        let socket = UnixListener::bind(socket_path)?;

        let (s, my_receiver) = mpsc::unbounded_channel();
        let (sub_sender, sr) = watch::channel(Message::default());

        Ok(Self {
            sender: Arc::new(s),
            my_receiver,
            sub_sender,
            receiver: Arc::new(sr),
            socket,
            buffer: Vec::new(),
        })
    }
    /// Read/write to the stream indefinitely. This should not return.
    ///
    /// Consider spawning this on another async task, or joining this with another future that runs indefinitely.
    // #[cfg_attr(feature = "tracing", ::tracing::instrument(level = "debug", skip_all))]
    pub async fn read_forever(&mut self) -> Result<(), Error> {
        let (stream, address) = self.socket.accept().await?;
        #[cfg(feature = "tracing")]
        tracing::debug!(
            "Halogen server received connection from address: {:?}",
            address.as_pathname()
        );

        let mut partial_line = String::new();
        // clear partial line when it is done
        let mut clear_line = false;

        loop {
            if clear_line {
                clear_line = false;
                partial_line.clear();
            }
            let mut current_data = Vec::with_capacity(2048);

            stream.readable().await?;

            loop {
                let mut buffer = [0; 2048];
                let read = stream.try_read(&mut buffer)?;
                current_data.copy_from_slice(&buffer);

                if read == 0 {
                    break;
                }
            }

            let decoded = match std::str::from_utf8(current_data.as_slice()) {
                Ok(s) => s,
                Err(e) => {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("Halogen server decoding error: {e}");
                    continue;
                }
            };

            let mut lines = decoded.lines().collect::<VecDeque<_>>();
            if !partial_line.is_empty() {
                // safety: I already broke the loop if it was empty, this should return one.
                let other_part = lines.pop_front().expect("Halogen server expected a partial line, received none! Please file a bug report!");
                partial_line.push_str(other_part);
                lines.push_front(partial_line.as_str());

                clear_line = true;
            } else if !decoded.ends_with('\n') {
                // it is incomplete, wait for more data to come through
                // safety: I already broke the loop if it was empty, this should return one.
                partial_line.push_str(lines.pop_back().expect("Halogen server expected a partial line, received none! Please file a bug report!"))
            }

            for line in lines {
                send_helper(line, &self.sub_sender).await?;
            }
        }

        // Err(Error::EarlyReturn)
    }
    pub fn send_message(&self, message: Message) -> Result<(), Error> {
        self.sender.send(message)?;
        Ok(())
    }
}

/// An internal helper function for the reading thingy
async fn send_helper(line: &str, sender: &watch::Sender<Message>) -> Result<(), Error> {
    let msg = Message::try_from_raw(line)?;
    sender.send_if_modified(|old| {
        if *old != msg {
            let _ = std::mem::replace(old, msg);
            return true;
        }

        false
    });
    Ok(())
}
