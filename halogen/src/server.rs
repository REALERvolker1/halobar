use crate::imports::*;

pub struct Server {
    pub socket_path: PathBuf,
    /// A Sender to send messages to the socket
    pub sender: Arc<ServerSender<Message>>,
    receiver: ServerReceiver<Message>,
}
impl Server {
    pub async fn new() -> Result<Self, Error> {
        let socket_path = crate::get_socket_path()?;
        let (s, receiver) = unbounded();

        Ok(Self {
            socket_path,
            sender: Arc::new(s),
            receiver,
        })
    }
}
