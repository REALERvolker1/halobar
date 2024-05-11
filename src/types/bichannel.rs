use crate::prelude::{error, mpsc, Arc};

/// A two-way mpmc channel.
///
/// TODO: Document more
#[derive(Debug)]
pub struct BiChannel<T, F> {
    pub context: String,
    pub sender: Arc<mpsc::Sender<T>>,
    pub receiver: mpsc::Receiver<F>,
}
impl<T, F> BiChannel<T, F> {
    /// Create a new two-way mpsc channel. The buffer is the number of messages it holds before applying backpressure,
    /// and the context is the string that it logs just in case of any errors during the course of its operation.
    pub fn new<S: Into<String>>(
        buffer: usize,
        first_context: Option<S>,
        second_context: Option<S>,
    ) -> (BiChannel<T, F>, BiChannel<F, T>) {
        let (sender1, receiver1) = mpsc::channel(buffer);
        let (sender2, receiver2) = mpsc::channel(buffer);

        (
            BiChannel {
                context: match first_context {
                    Some(s) => s.into(),
                    None => "None".to_owned(),
                },
                sender: Arc::new(sender1),
                receiver: receiver2,
            },
            BiChannel {
                context: match second_context {
                    Some(s) => s.into(),
                    None => "None".to_owned(),
                },
                sender: Arc::new(sender2),
                receiver: receiver1,
            },
        )
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
