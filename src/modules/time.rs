use chrono::format::StrftimeItems;
use halobar_config::config_struct;
use tokio::sync::watch;

use super::*;

config_struct! {
    [Time]
    format: String = "%I:%M:%S %P".to_owned(),
    format_alt: String = "%a, %m/%d @ %I:%M:%S %P".to_owned(),
    interval: u64 = 750,
}

pub struct Time {
    format: String,
    format_alt: String,
    interval: Duration,
    channel: BiChannel<String, Event>,
    state: FormatState,
    last_event: RwLock<Event>,
}
impl BackendModule for Time {
    type Input = ();
    type Config = TimeKnown;
    type Error = TimeError;
    async fn new(
        _input: Self::Input,
        config: Self::Config,
    ) -> Result<(Self, BiChannel<Event, String>), Self::Error> {
        let (channel, yours) =
            BiChannel::new(5, Some("Time module".into()), Some("Time receiver".into()));

        let me = Self {
            format: config.format,
            format_alt: config.format_alt,
            interval: Duration::from_millis(config.interval),
            channel,
            state: FormatState::default(),
            last_event: RwLock::const_new(Event::default()),
        };
        Ok((me, yours))
    }
    async fn run(&mut self) -> Result<(), Self::Error> {
        let mut receiver = self
            .channel
            .receiver
            .take()
            .expect("Time receiver was not found!");
        let event_receiver = async {
            while let Some(ev) = receiver.recv().await {
                self.receive_event(ev).await?;
            }
            rok![(), TimeError]
        };
        let operation = async {
            // TODO: Optimize this
            loop {
                let time = chrono::Local::now();
                let format = match self.state {
                    FormatState::Normal => StrftimeItems::new(&self.format),
                    FormatState::Alternate => StrftimeItems::new(&self.format_alt),
                };

                let items = time.format_with_items(format);

                join!(
                    self.channel.send(items.to_string()),
                    tokio::time::sleep(self.interval)
                );
            }
        };

        let (_, r) = tokio::join!(operation, event_receiver);
        r?;
        Ok(())
    }
    async fn receive_event(&self, event: Event) -> Result<(), Self::Error> {
        match event {
            Event::Click | Event::MiddleClick | Event::RightClick => {
                let mut write = self.last_event.write().await;
                *write = event
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TimeError {
    #[error("Invalid strftime format: {0}")]
    InvalidStrftime(String),
}
