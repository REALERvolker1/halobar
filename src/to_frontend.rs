//! Various functions that are an abstraction over what the frontend will eventually be like.

use crate::prelude::*;

pub type FrontendSender<T> = Arc<mpsc::UnboundedSender<T>>;

/// A sender/receiver that makes sure messages go to the proper places.
pub struct FrontendMux {
    pub message_sender: FrontendSender<Message>,
    message_receiver: mpsc::UnboundedReceiver<Message>,
    event_sender: FrontendSender<Event>,
    event_receiver: mpsc::UnboundedReceiver<Event>,
}
impl FrontendMux {
    /// Create a new `FrontendMux`.
    /// This must be called once, upon initialization at startup.
    pub fn new() -> Self {
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            message_sender: Arc::new(message_sender),
            message_receiver,
            event_sender: Arc::new(event_sender),
            event_receiver,
        }
    }

    /// Run this muxer. This should not return.
    ///
    /// It only has a result type so it picks up eyre info if anything goes wrong.
    ///
    /// TODO: Add event handler
    pub async fn run(self) -> ! {
        let mut event_receiver = self.event_receiver;
        let mut message_receiver = self.message_receiver;

        // TODO: Add event handler
        loop {
            select! {
                maybe_event = event_receiver.recv() => {
                    let event = maybe_event.expect("halobar event receiver closed unexpectedly! Please file a bug report!");

                    debug!("Received event {event:?}");
                }

                maybe_message = message_receiver.recv() => {
                    let message = maybe_message.expect("halobar message receiver closed unexpectedly! Please file a bug report!");

                    debug!("Received message: {message:?}");
                }
            }
        }
    }
}
