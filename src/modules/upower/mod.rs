pub mod types;
mod xmlgen;

use std::sync::atomic::AtomicBool;

use super::*;
use types::*;
use xmlgen::{display_device::DeviceProxy, keyboard::KbdBacklightProxy, upower::UPowerProxy};
use zbus::{
    proxy::{CacheProperties, PropertyStream},
    Connection,
};

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

        self.calc_brightness_percent(current)
            .ok_or_else(|| zbus::Error::Failure(format!("Percentage int is too big!")))
    }

    pub fn calc_brightness_percent(&self, brightness: i32) -> Option<Percentage> {
        let current_percent = (brightness * 100) / self.max_brightness;

        Percentage::try_new(current_percent.unsigned_abs() as u8).ok()
    }

    pub async fn brightness_percent_listener(
        &self,
        data_channel: Arc<mpsc::UnboundedSender<ModuleData>>,
    ) -> R<()> {
        let mut stream = self.keyboard.receive_brightness_changed().await?;

        while let Some(b) = stream.next().await {
            let new_brightness = b.args()?.value;
            let percent = self
                .calc_brightness_percent(new_brightness)
                .ok_or_else(|| {
                    zbus::Error::Failure(format!("Percentage int {new_brightness} is too big!"))
                })?;

            data_channel.send(ModuleData::new(Data::Upower(
                UpowerData::KeyboardBrightnessPercentage(percent),
            )))?;
        }

        Ok(())
    }

    pub async fn brightness_value_listener(
        &self,
        data_channel: Arc<mpsc::UnboundedSender<ModuleData>>,
    ) -> R<()> {
        let mut stream = self.keyboard.receive_brightness_changed().await?;

        while let Some(b) = stream.next().await {
            let new_brightness = b.args()?.value;

            data_channel.send(ModuleData::new(Data::Upower(
                UpowerData::KeyboardBrightness(new_brightness),
            )))?;
        }

        Ok(())
    }
}

/// This is here because I need to always know this bool to determine some other states.
///
/// It also needs to be shared between threads.
static DEVICE_ON_BATTERY: AtomicBool = AtomicBool::new(false);
/// A getter for the battery prop but as a real bool
fn on_battery() -> bool {
    DEVICE_ON_BATTERY.load(::std::sync::atomic::Ordering::SeqCst)
}
fn set_on_battery(on_battery: bool) {
    DEVICE_ON_BATTERY.store(on_battery, std::sync::atomic::Ordering::SeqCst)
}

#[derive(Debug)]
struct Upower<'c> {
    conn: &'c Connection,

    upower: UPowerProxy<'c>,
    device: DeviceProxy<'c>,
    keyboard: OnceCell<Keyboard<'c>>,

    /// I use these containers for my own convenience.
    /// These hold the data that already was sent, but that is updated.
    props: RefCell<Vec<UpowerData>>,
}
impl<'c> Upower<'c> {
    pub async fn new(conn: &'c Connection, device_path: String) -> R<Self> {
        let upower = UPowerProxy::builder(conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let (device, on_battery) = try_join!(
            async {
                let mut builder = DeviceProxy::builder(conn).cache_properties(CacheProperties::No);

                if !device_path.is_empty() {
                    builder = builder.path(device_path)?;
                }

                builder.build().await
            },
            upower.on_battery()
        )?;

        set_on_battery(on_battery);

        Ok(Self {
            conn,
            upower,
            device,
            keyboard: OnceCell::new(),
            props: RefCell::new(Vec::new()),
        })
    }

    /// This is a getter that ensures the keyboard proxy is init.
    pub async fn get_keyboard(&'c self) -> zbus::Result<&'c Keyboard<'c>> {
        match self.keyboard.get() {
            Some(k) => Ok(k),
            None => {
                let keyboard = Keyboard::new(&self.conn).await?;
                self.keyboard.set(keyboard).map_err(|_| zbus::Error::Failure("Failed to initialize keyboard, failed to get field, but field was already set after initializing proxies!".to_owned()))?;
                self.keyboard.get().ok_or(zbus::Error::Failure(
                    "Failed to initialize keyboard, field remained empty after init!".to_owned(),
                ))
            }
        }
    }

    /// Resolve/reject a request. This mutates a reference.
    pub async fn fulfill_initial_request(&'c self, request: &mut Request) -> R<()> {
        let discriminant = match request {
            Request::Request(RequestField::Upower(d)) => d,
            _ => {
                request.reject_invalid();
                return Ok(());
            }
        };

        // I may already have it cached
        {
            let props = self.props.borrow();

            for data in props.iter() {
                let data_type: UpowerDataDiscriminants = data.try_into()?;

                if *discriminant == data_type {
                    request.resolve(ModuleData::new(Data::Upower(data.clone())));
                    return Ok(());
                }
            }
        }

        macro_rules! arm {
            ($( $enum_arm:ident => $prop_getter:expr),+$(,)?) => {
                match discriminant {
                    $(
                        UpowerDataDiscriminants::$enum_arm => {
                            match $prop_getter.await {
                                Ok(prop) => {
                                    // listener_set.push($listener_future);
                                    {
                                        let mut borrowed = self.props.borrow_mut();
                                        borrowed.push(UpowerData::$enum_arm(prop.clone()));
                                    }
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

        let data = arm! {
            Energy => self.device.energy(),
            EnergyRate => self.device.energy_rate(),
            Icon => self.device.icon_name(),
            Percentage => self.device.percentage(),
            State => self.device.state(),
            Time => async {
                let time = if on_battery() {
                    self.device.time_to_empty().await
                } else {
                    self.device.time_to_full().await
                }?;

                let dura = Duration::from_secs(time.unsigned_abs());

                Ok::<Duration, zbus::Error>(dura)
            },
            DeviceType => self.device.type_(),
            WarningLevel => self.device.warning_level(),

            CriticalAction => self.upower.get_critical_action(),
            KeyboardBrightnessPercentage => async {
                self.get_keyboard().await?.get_brightness_percent().await
            },
            KeyboardBrightness => async {
                self.get_keyboard().await?.keyboard.get_brightness().await
            },
            KeyboardBrightnessMax => async {
                self.get_keyboard().await?.keyboard.get_max_brightness().await
            },
        };

        match data {
            Some(d) => request.resolve(ModuleData::new(Data::Upower(d))),
            None => request.reject(ProviderError::QueryError),
        }

        // {
        //     let mut empty_stream = self.device.receive_time_to_empty_changed().await;
        //     let mut full_stream = self.device.receive_time_to_full_changed().await;
        //     Box::new(async move {

        //     loop {
        //         let new_time = select! {
        //             Some(empty) = empty_stream.next() => {
        //                 if self.on_battery() {
        //                     continue;
        //                 }

        //                 empty.get().await?
        //             }

        //             Some(full) = full_stream.next() => {
        //                 if !self.on_battery() {
        //                     continue;
        //                 }

        //                 full.get().await?
        //             }
        //         };

        //         channel.send(ModuleData::new(Data::Upower(UpowerData::Time(Duration::from_secs(new_time.unsigned_abs())))))?;

        //     }
        //     // Ok::<(), Report>(())
        // })},

        Ok(())
    }
}

/// This is literally just here so I can have the Upower struct just reference
/// the connection so I don't have to mess with self-referential structs
pub struct UpowerMod {
    conn: Connection,
    channel: BiChannel<ModuleData, Event>,
}
impl<'c> UpowerMod {
    async fn listen_to_stream<T>(&'c self, mut stream: PropertyStream<'c, T>) -> R<()>
    where
        T: std::any::Any,
    {
        while let Some(change) = stream.
        Ok(())
    }
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

        let upower = Upower::new(&conn, my_config.device_path).await?;

        for mut data_request in requests.into_iter() {
            let mut requests = data_request
                .data_fields
                .iter_mut()
                .map(|req| upower.fulfill_initial_request(req))
                .collect::<FuturesUnordered<_>>();

            while let Some(result) = requests.next().await {
                result?;
            }
        }

        // I need to loop over all the requested types one more time because
        // I could not return futures referencing self in upower.fulfill_initial_request()
        // and I want this to be single-threaded.

        {
            let props = upower
                .props
                .borrow()
                .iter()
                .filter_map(|prop| {
                    let discriminant: UpowerDataDiscriminants = prop.try_into().ok()?;
                    Some(discriminant)
                })
                .map(|disc| disc);
        }

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
