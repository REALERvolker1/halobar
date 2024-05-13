pub mod types;
mod xmlgen;

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

struct Proxies<'c> {
    conn: &'c Connection,
    upower: UPowerProxy<'c>,
    device: DeviceProxy<'c>,
    keyboard: Option<KbdBacklightProxy<'c>>,
}
impl<'c> Proxies<'c> {
    pub async fn new(conn: &'c Connection, device_path: String) -> zbus::Result<Self> {
        let upower = UPowerProxy::builder(conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let device = {
            let mut builder = DeviceProxy::builder(conn).cache_properties(CacheProperties::No);

            if !device_path.is_empty() {
                builder = builder.path(device_path)?;
            }

            builder.build().await?
        };

        Ok(Self {
            conn,
            upower,
            device,
            keyboard: None,
        })
    }

    /// ensure the keyboard proxy is init. returns if it was or not.
    pub async fn ensure_keyboard(&mut self) -> zbus::Result<bool> {
        if self.keyboard.is_some() {
            return Ok(true);
        }

        let proxy = KbdBacklightProxy::builder(self.conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        self.keyboard.replace(proxy);

        Ok(false)
    }
}

pub struct Upower {
    conn: Connection,
    channel: BiChannel<ModuleData, Event>,
    /// I use these containers for my own convenience.
    /// These hold the data that already was sent, but that is updated.
    props: Vec<UpowerData>,
}
impl ModuleDataProvider for Upower {
    type ServerConfig = UpowerConfig;
    async fn main(
        config: Self::ServerConfig,
        mut requests: Vec<DataRequest>,
        yield_channel: mpsc::UnboundedSender<ModuleYield>,
    ) -> R<()> {
        let my_config = config.into_known();

        let conn = crate::globals::get_zbus_system().await?;

        let proxies = Proxies::new(&conn, my_config.device_path).await?;

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
    Percentage(u8),
    State(BatteryState),
    Time(Duration),
    DeviceType(DeviceType),
    WarningLevel(WarningLevel),

    CriticalAction(CriticalAction),
    KeyboardBrightnessPercentage(u8),
    KeyboardBrightness(i32),
    KeyboardBrightnessMax(i32),
}
