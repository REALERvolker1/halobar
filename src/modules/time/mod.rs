use chrono::format::StrftimeItems;

use super::*;

config_struct! {
    @known {Clone}
    @config {Clone}
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
                self.channel.send(Self::module_data(self.tick()))
            );
        }
    }
}
impl BackendModule for Time {
    type Input = TimeConfig;
    const MODULE_TYPE: ModuleType = ModuleType::Time;
    async fn run(
        module_id: ModuleId,
        input: Self::Input,
        yield_sender: Arc<mpsc::UnboundedSender<ModuleYield>>,
    ) -> R<bool> {
        let config = input.into_known();
        let (channel, yielded) = BiChannel::new(15, Some("Time module"), Some("Time receiver"));

        let me = Self {
            format: config.format,
            interval: Duration::from_millis(config.interval_ms),
            channel,
        };

        let yielded = ModuleYield {
            id: module_id,
            data_output: OutputType::Loop(yielded),
            module_type: Self::MODULE_TYPE,
        };

        yield_sender.send(yielded)?;

        me.listen().await;

        return Ok(false);
    }
}
