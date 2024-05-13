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
    channel: BiChannel<ModuleData, Event>,
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
            .map(|d| async {
                let mod_id = d.specific_target.clone().unwrap();
                let send = self.channel.send(d).await;

                (send, mod_id)
            })
            .collect::<FuturesUnordered<_>>();

        while let Some((was_sent, module)) = data.next().await {
            if !was_sent {
                self.format_strings.remove(&module);
            }
        }
    }
}

impl ModuleDataProvider for Time {
    type ServerConfig = TimeConfig;

    async fn main(
        config: Self::ServerConfig,
        mut requests: Vec<DataRequest>,
        yield_channel: mpsc::UnboundedSender<ModuleYield>,
    ) -> R<()> {
        let my_config = config.into_known();

        let (channel, subscription) = BiChannel::new(24);

        let mut me = Self {
            format_strings: AHashMap::new(),
            interval: Duration::from_millis(my_config.interval_ms),
            channel,
        };

        let time = Self::get_time();

        for data_request in requests.iter_mut() {
            for request in data_request.data_fields.iter_mut() {
                match request {
                    Request::Request(RequestField::Time(strf)) => {
                        let stime = Strftime(strf.clone());

                        request.resolve(ModuleData {
                            specific_target: Some(data_request.id.clone()),
                            content: Data::Time(TimeData(stime.format_time(&time))),
                        });

                        me.format_strings.insert(data_request.id.clone(), stime);
                    }
                    _ => request.reject_invalid(),
                }
            }
        }

        let yields = ModuleYield {
            subscription: Some(subscription),
            fulfilled_requests: requests,
        };

        yield_channel.send(yields)?;

        loop {
            join!(tokio::time::sleep(me.interval), me.tick());
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
