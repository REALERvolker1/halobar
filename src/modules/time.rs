use chrono::format::StrftimeItems;
use halobar_config::config_struct;
use tokio::sync::watch;

use super::*;

config_struct! {
    [Time]
    format: String = "%I:%M:%S %P".to_owned(),
    format_alt: String = "%a, %m/%d @ %I:%M:%S %P".to_owned(),
    interval: u64 = 1000,
}

pub struct Time {
    format: String,
    format_alt: String,
    interval: Duration,
    channel: BiChannel<String, Event>,
    state: RwLock<FormatState>,
}
impl BackendModule for Time {
    type Input = ();
    type Config = TimeKnown;
    type Error = TimeError;
    #[instrument(level = "debug", skip_all)]
    async fn new(
        _input: Self::Input,
        config: Self::Config,
    ) -> Result<(Self, BiChannel<Event, String>), Self::Error> {
        let (channel, yours) =
            BiChannel::new(5, Some("Time module".into()), Some("Time receiver".into()));

        // let (state_sender, state_receiver) = watch::channel(FormatState::default());

        let me = Self {
            format: config.format,
            format_alt: config.format_alt,
            interval: Duration::from_millis(config.interval),
            channel,
            state: RwLock::new(FormatState::Normal),
        };
        Ok((me, yours))
    }
    async fn run(&mut self) -> Result<(), Self::Error> {
        let mut receiver = self
            .channel
            .get_receiver()
            .expect("Time receiver was not found!");
        let event_receiver = async {
            while let Some(ev) = receiver.recv().await {
                self.receive_event(ev).await?;
            }
            rok![(), TimeError]
        };
        let operation = async {
            let mut items = String::new();
            // TODO: Optimize this
            loop {
                join!(
                    self.channel.send(items.clone()),
                    tokio::time::sleep(self.interval),
                    async {
                        let format = match *self.state.read().await {
                            FormatState::Normal => StrftimeItems::new(&self.format),
                            FormatState::Alternate => StrftimeItems::new(&self.format_alt),
                        };

                        let time = chrono::Local::now();
                        items = time.format_with_items(format).to_string();
                    }
                );
            }
        };

        let (_, r) = tokio::join!(operation, event_receiver);
        r?;
        Ok(())
    }
    #[instrument(level = "debug", skip(self))]
    async fn receive_event(&self, event: Event) -> Result<(), Self::Error> {
        match event {
            Event::Click | Event::MiddleClick | Event::RightClick => {
                let mut s = self.state.write().await;
                s.next();
            }

            _ => {}
        }
        Ok(())
    }
    #[inline]
    fn module_type() -> ModuleType {
        ModuleType::Loop
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Time error!")]
pub struct TimeError;
