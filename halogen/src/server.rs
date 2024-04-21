use monoio::net::UnixListener;

use crate::imports::*;

pub struct Server {
    /// A Sender to send messages to the socket
    pub sender: Arc<ServerSender<Message>>,
    receiver: ServerReceiver<Message>,
    socket: UnixListener,
    buffer: Vec<u8>,
}
impl Server {
    /// Create a new [`Server`] to write to the socket
    #[cfg_attr(feature = "tracing", ::tracing::instrument(level = "debug", skip_all))]
    pub async fn new() -> Result<Self, Error> {
        let socket_path = crate::get_socket_path()?;
        // let stream = UnixStream::connect(&socket_path).await?;
        let socket = UnixListener::bind(socket_path)?;

        let (s, receiver) = unbounded();

        Ok(Self {
            sender: Arc::new(s),
            receiver,
            socket,
            buffer: Vec::new(),
        })
    }
    /// Read/write to the stream indefinitely. This should not return.
    ///
    /// Consider spawning this on another async task, or joining this with another future that runs indefinitely.
    #[cfg_attr(feature = "tracing", ::tracing::instrument(level = "debug", skip_all))]
    pub async fn read_forever(&mut self) -> Result<(), Error> {
        while let Some(value) = self.socket.next().await {
            let (mut stream, address) = value?;

            let mut buf = Vec::with_capacity(8192);

            loop {
                let (maybe_len, buffer) = stream.read(buf).await;
                if maybe_len? == 0 {
                    break;
                }
                println!("Received bytes: {:?}", buffer.as_slice());
                buf = buffer;
            }
        }
        Err(Error::EarlyReturn)
    }
}
// impl Stream for Server {
//     type Item = Message;
//     fn next(&mut self) -> impl futures_util::Future<Output = Option<Self::Item>> {
//         futures_util::po
//     }
// }
