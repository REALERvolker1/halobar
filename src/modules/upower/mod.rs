pub mod types;
mod xmlgen;

use std::sync::atomic::AtomicBool;

use super::*;
use types::*;
use xmlgen::{display_device::DeviceProxy, keyboard::KbdBacklightProxy, upower::UPowerProxy};
use zbus::{proxy::CacheProperties, Connection};

config_struct! {
    @known {Clone}
    @config {Clone}
    [Upower]
    device_path: String = String::new(),
}

#[derive(Debug)]
struct Keyboard<'c> {
    conn: &'c Connection,
    keyboard: KbdBacklightProxy<'c>,
    max_brightness: i32,
}
impl<'c> Keyboard<'c> {
    pub async fn new(conn: &'c Connection) -> zbus::Result<Self> {
        let keyboard = KbdBacklightProxy::builder(conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let max_brightness = keyboard.get_max_brightness().await?;

        Ok(Self {
            conn,
            keyboard,
            max_brightness,
        })
    }

    pub async fn get_brightness_percent(&self) -> zbus::Result<Percentage> {
        let current = self.keyboard.get_brightness().await?;

        self.calc_brightness_percent(current).ok_or_else(|| zbus::Error::Failure(format!("Percentage int is too big!")))
    }

    pub fn calc_brightness_percent(&self, brightness: i32) -> Option<Percentage> {
        let current_percent = (brightness * 100) / self.max_brightness;

        Percentage::try_new(current_percent.unsigned_abs() as u8).ok()
    }

    pub async fn brightness_percent_listener(&self, data_channel: Arc<mpsc::UnboundedSender<ModuleData>>) -> R<()> {
        let mut stream = self.keyboard.receive_brightness_changed().await?;

        while let Some(b) = stream.next().await {
            let new_brightness = b.args()?.value;
            let percent = self.calc_brightness_percent(new_brightness).ok_or_else(|| zbus::Error::Failure(format!("Percentage int {new_brightness} is too big!")))?;

            data_channel.send(ModuleData::new(Data::Upower(UpowerData::KeyboardBrightnessPercentage(percent))))?;
        }

        Ok(())
    }

    pub async fn brightness_value_listener(&self, data_channel: Arc<mpsc::UnboundedSender<ModuleData>>) -> R<()> {
        let mut stream = self.keyboard.receive_brightness_changed().await?;

        while let Some(b) = stream.next().await {
            let new_brightness = b.args()?.value;

            data_channel.send(ModuleData::new(Data::Upower(UpowerData::KeyboardBrightness(new_brightness))))?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Upower<'c> {
    conn: &'c Connection,

    upower: UPowerProxy<'c>,
    device: DeviceProxy<'c>,
    keyboard: OnceCell<Keyboard<'c>>,

    /// This is here because I need to always know this bool to determine some other states.
    on_battery: AtomicBool,

    /// I use these containers for my own convenience.
    /// These hold the data that already was sent, but that is updated.
    props: Vec<UpowerData>,
}
impl<'c> Upower<'c> {
    pub async fn new(conn: &'c Connection, device_path: String) -> R<Self> {
        let upower = UPowerProxy::builder(conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let (device, on_battery) = try_join!(async {
            let mut builder = DeviceProxy::builder(conn).cache_properties(CacheProperties::No);

            if !device_path.is_empty() {
                builder = builder.path(device_path)?;
            }

            builder.build().await
        }, upower.on_battery())?;




        Ok(Self {
            conn,
            upower,
            device,
            keyboard: OnceCell::new(),
            on_battery: AtomicBool::new(on_battery),
            props: Vec::new(),
        })
    }

    /// This is a getter that ensures the keyboard proxy is init.
    pub async fn get_keyboard(&'c self) -> zbus::Result<&'c Keyboard<'c>> {
        match self.keyboard.get() {
            Some(k) => Ok(k),
            None => {
                let keyboard = Keyboard::new(&self.conn).await?;
                self.keyboard.set(keyboard).map_err(|_| zbus::Error::Failure("Failed to initialize keyboard, failed to get field, but field was already set after initializing proxies!".to_owned()))?;
                self.keyboard.get().ok_or(zbus::Error::Failure("Failed to initialize keyboard, field remained empty after init!".to_owned()))
            }
        }
    }

    /// A getter for the battery prop but as a real bool
    pub fn on_battery(&self) -> bool {
        self.on_battery.load(::std::sync::atomic::Ordering::SeqCst)
    }

    /// Resolve/reject a request. This mutates a reference.
    pub async fn fulfill_initial_request(&'c self, request: &mut Request, listener_set: &'c mut FuturesUnordered<R<()>>, channel: Arc<mpsc::UnboundedSender<ModuleData>>) -> R<()> {
        let discriminant = match request {
            Request::Request(RequestField::Upower(d)) => d,
            _ => {
                request.reject_invalid();
                return Ok(())
            }
        };

        // I already have it cached
        for data in self.props.iter() {
            let data_type: UpowerDataDiscriminants = data.try_into()?;

            if *discriminant == data_type {
                request.resolve(ModuleData::new(Data::Upower(data.clone())));
                return Ok(())
            }
        }

        macro_rules! arm {
            ($( $enum_arm:ident => $prop_getter:expr, $listener_future: expr ),+$(,)?) => {
                match discriminant {
                    $(
                        UpowerDataDiscriminants::$enum_arm => {
                            match $prop_getter.await {
                                Ok(prop) => {
                                    listener_set.push($listener_future);

                                    Some(UpowerData::$enum_arm(prop))
                                }
                                Err(e) => {
                                    warn!("Error getting data for request {}: {e}", stringify!($enum_arm));
                                    None
                                }
                            }

                        }
                    )+
                }
            };
        }

        macro_rules! prop_stream {
            ($listener:expr => $enum_arm:ident) => {
                async {
                    let mut stream = $listener.await;
                    while let Some(d) = stream.next().await {
                        let data = d.get().await?;

                        channel.send(ModuleData::new(Data::Upower(UpowerData::$enum_arm(data))))?;
                    }

                    Ok::<(), Report>(())
                }
            };
        }

        let data = arm! {
            Energy => self.device.energy(), prop_stream!(self.device.receive_energy_changed() => Energy),
            EnergyRate => self.device.energy_rate(), prop_stream!(self.device.receive_energy_rate_changed() => EnergyRate),
            Icon => self.device.icon_name(), prop_stream!(self.device.receive_icon_name_changed() => EnergyRate),
            Percentage => self.device.percentage(), prop_stream!(self.device.receive_percentage_changed() => EnergyRate),
            State => self.device.state(), prop_stream!(self.device.receive_state_changed() => EnergyRate),
            Time => async {
                let time = if self.on_battery() {
                    self.device.time_to_empty().await
                } else {
                    self.device.time_to_full().await
                }?;

                let dura = Duration::from_secs(time.unsigned_abs());

                Ok::<Duration, zbus::Error>(dura)
            }, async {
                let mut empty_stream = self.device.receive_time_to_empty_changed().await;
                let mut full_stream = self.device.receive_time_to_full_changed().await;

                loop {
                    let new_time = select! {
                        Some(empty) = empty_stream.next() => {
                            if self.on_battery() {
                                continue;
                            }

                            empty.get().await?
                        }

                        Some(full) = full_stream.next() => {
                            if !self.on_battery() {
                                continue;
                            }

                            full.get().await?
                        }
                    };

                    channel.send(ModuleData::new(Data::Upower(UpowerData::Time(Duration::from_secs(time.unsigned_abs()))))).await?;

                    Ok::<(), Report>(())
                }
            },
            DeviceType => self.device.type_(), prop_stream!(self.device.receive_type_changed() => EnergyRate),
            WarningLevel => self.device.warning_level(), prop_stream!(self.device.receive_warning_level_changed() => EnergyRate),

            CriticalAction => self.upower.get_critical_action(), prop_stream!(self.upower.receive_critical_action_changed() => EnergyRate),
            KeyboardBrightnessPercentage => async {
                self.get_keyboard().await?.get_brightness_percent().await
            }, async {
                self.get_keyboard().await?.brightness_percent_listener().await
            },
            KeyboardBrightness => async {
                self.get_keyboard().await?.keyboard.get_brightness().await
            }, async {
                self.get_keyboard().await?.brightness_value_listener().await
            },
            KeyboardBrightnessMax => async {
                self.get_keyboard().await?.keyboard.get_max_brightness().await
            }, async {
                // This is static, but I still have to write it this way because this is a dynamic block
                Ok::<(), Report>(())
            }
        };

        match data {
            Some(d) => request.resolve(ModuleData::new(Data::Upower(d))),
            None => request.reject(ProviderError::QueryError),
        }

        Ok(())
    }
}


/// This is literally just here so I can have the Upower struct just reference
/// the connection so I don't have to mess with self-referential structs
pub struct UpowerMod {
    conn: Connection,
    channel: BiChannel<ModuleData, Event>,
}
impl ModuleDataProvider for UpowerMod {
    type ServerConfig = UpowerConfig;
    async fn main(
        config: Self::ServerConfig,
        mut requests: Vec<DataRequest>,
        yield_channel: mpsc::UnboundedSender<ModuleYield>,
    ) -> R<()> {
        let my_config = config.into_known();

        let conn = crate::globals::get_zbus_system().await?;

        let mut upower = Upower::new(&conn, my_config.device_path).await?;

        let mut requests_iter = requests.into_iter();

        let first_request = requests_iter
            .next()
            .expect("Info providers cannot be given empty request vectors!");

            // TODO: This design pattern was buggy. Refactor
        let mut requests = first_request.union(requests_iter);

        // I do all the subscription waiting on the same thread to save you system resources. You're welcome.
        // let mut listener_futures = FuturesUnordered::new();
        let mut requested_fields = AHashSet::new();

        for request in requests.it

        let request_futures = requests
            .into_iter()
            .map(|mut request| match request {
                Request::Request(RequestField::Upower(disc)) => {
                    requested_fields.insert(disc);

                    match disc {
                        UpowerDataDiscriminants::Energy => Box::new(async {
                            let prop = proxies.device.energy().await?;

                            request
                                .resolve(ModuleData::new(Data::Upower(UpowerData::Energy(prop))));

                            Ok::<Request, zbus::Error>(request)
                        }),
                        UpowerDataDiscriminants::EnergyRate => Box::new(async {
                            let prop = proxies.device.energy_rate().await?;

                            request.resolve(ModuleData::new(Data::Upower(UpowerData::EnergyRate(
                                prop,
                            ))));

                            Ok::<Request, zbus::Error>(request)
                        }),
                    }
                    // Ok::<(), zbus::Error>(())
                }
                _ => request.reject_invalid(),
            })
            .collect::<FuturesUnordered<Box<dyn std::future::Future<zbus::Result<Request>>>>>();

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, strum_macros::EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
pub enum UpowerData {
    Energy(f64),
    EnergyRate(f64),
    Icon(String),
    Percentage(Percentage),
    State(BatteryState),
    Time(Duration),
    DeviceType(DeviceType),
    WarningLevel(WarningLevel),

    CriticalAction(CriticalAction),
    KeyboardBrightnessPercentage(Percentage),
    KeyboardBrightness(i32),
    KeyboardBrightnessMax(i32),
}
