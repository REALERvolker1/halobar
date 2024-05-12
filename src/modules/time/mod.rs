use chrono::{format::StrftimeItems, DateTime, Local};

use super::*;

config_struct! {
    @known {Clone}
    @config {Clone}
    [Time]
    // format: String = "%I:%M:%S %P".to_owned(),
    // format_alt: String = "%a, %m/%d @ %I:%M:%S %P".to_owned(),
    interval_ms: u64 = 1000,
}

/// A newtype struct so I don't get confused when referring to types
struct Strftime(String);
impl<'a> Strftime {
    pub fn consume(self) -> String {
        self.0
    }
    pub fn as_str(&'a self) -> &'a str {
        self.0.as_str()
    }
    pub fn format_time(&self, time: &DateTime<chrono::Local>) -> String {
        time.format(&self.0).to_string()
    }
}

pub struct Time {
    format_strings: AHashMap<ModuleId, Strftime>,
    // format_alt: String,
    interval: Duration,
    interface: ProviderData,
    // state: Mutex<FormatState>,
}
impl Time {
    fn get_time() -> DateTime<Local> {
        Local::now()
    }

    async fn tick(&mut self) {
        let time = Self::get_time();

        let mut data = self
            .format_strings
            .iter()
            .map(|(id, stime)| (id.clone(), stime.format_time(&time)))
            .map(|(id, timestr)| ModuleData {
                specific_target: Some(id),
                content: Data::Time(TimeData(timestr)),
            })
            .map(|d| {
                let mod_id = d.specific_target.clone().unwrap();
                let send = self.interface.data_sender.send(d);

                (send, mod_id)
            })
            .collect::<Box<[_]>>();

        for (was_sent, module) in data.into_iter() {
            if was_sent.is_err() {
                self.format_strings.remove(&module);
            }
        }
    }
}
impl ModuleDataProvider for Time {
    type ServerConfig = TimeConfig;

    async fn init(config: Self::ServerConfig, interface: ProviderData) -> R<Self> {
        let my_config = config.into_known();

        Ok(Self {
            format_strings: AHashMap::new(),
            interval: Duration::from_millis(my_config.interval_ms),
            interface,
        })
    }

    async fn process_data_requests(&mut self, requests: Vec<&mut DataRequest>) -> R<()> {
        let time = Self::get_time();

        for request in requests {
            for (strf, initial) in request.data_fields.iter_mut() {
                let stime = Strftime(strf.clone());

                initial.replace(ModuleData {
                    specific_target: Some(request.id.clone()),
                    content: Data::Time(TimeData(stime.format_time(&time))),
                });

                self.format_strings.insert(request.id.clone(), stime);
            }
        }

        Ok(())
    }

    async fn run(mut self) -> ! {
        loop {
            join!(tokio::time::sleep(self.interval), self.tick());
        }
    }
}
// impl BackendModule for Time {
//     type Input = TimeConfig;
//     const MODULE_TYPE: ModuleType = ModuleType::Time;
//     async fn run(
//         module_id: ModuleId,
//         input: Self::Input,
//         yield_sender: Arc<mpsc::UnboundedSender<ModuleYield>>,
//     ) -> R<bool> {
//         let config = input.into_known();
//         let (channel, yielded) = BiChannel::new(15, Some("Time module"), Some("Time receiver"));

//         let me = Self {
//             format: config.format,
//             interval: Duration::from_millis(config.interval_ms),
//             channel,
//         };

//         let yielded = ModuleYield {
//             id: module_id,
//             data_output: OutputType::Loop(yielded),
//             module_type: Self::MODULE_TYPE,
//         };

//         yield_sender.send(yielded)?;

//         me.listen().await;

//         return Ok(false);
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeData(pub String);
