use chrono::format::StrftimeItems;
use halobar_config::config_struct;

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
    pub state: FormatState,
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
        };
        Ok((me, yours))
    }
    async fn run(self) -> Result<(), Self::Error> {
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
    }
    async fn receive_event(&mut self, event: Event) -> Result<(), Self::Error> {
        // TODO: Make this configurable
        match event {
            Event::Click | Event::MiddleClick | Event::RightClick => self.state.next(),
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
