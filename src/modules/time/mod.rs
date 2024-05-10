use chrono::format::StrftimeItems;

use super::*;

config_struct! {
    [Time]
    format: String = "%I:%M:%S %P".to_owned(),
    // format_alt: String = "%a, %m/%d @ %I:%M:%S %P".to_owned(),
    interval_ms: u64 = 1000,
}

pub struct Time {
    format: String,
    // format_alt: String,
    interval: Duration,
    channel: BiChannel<ModuleData, Event>,
    // state: Mutex<FormatState>,
}
impl Time {
    fn tick(&self) -> String {
        let items = StrftimeItems::new(&self.format);

        let time = chrono::Local::now();

        return time.format_with_items(items).to_string();
    }
    async fn listen(self) -> ! {
        loop {
            join!(
                tokio::time::sleep(self.interval),
                self.channel.send(ModuleData::Time(self.tick()))
            );
        }
    }
}
impl BackendModule for Time {
    type Input = TimeKnown;
    const MODULE_TYPE: ModuleType = ModuleType::Time;
    async fn run(
        input: Self::Input,
        yield_sender: Arc<mpsc::UnboundedSender<(OutputType, ModuleType)>>,
    ) -> R<bool> {
        let (channel, yielded) = BiChannel::new(15, Some("Time module"), Some("Time receiver"));

        let me = Self {
            format: input.format,
            interval: Duration::from_millis(input.interval_ms),
            channel,
        };

        yield_sender.send((OutputType::Loop(yielded), Self::MODULE_TYPE))?;

        me.listen().await;

        return Ok(false);
    }
}
