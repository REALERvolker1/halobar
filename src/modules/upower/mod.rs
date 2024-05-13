pub mod types;
mod xmlgen;

use super::*;
use types::*;
use xmlgen::{display_device::DeviceProxy, upower::UPowerProxy};
use zbus::{proxy::CacheProperties, Connection};

config_struct! {
    @known {Clone}
    @config {Clone}
    [Upower]
    device_path: String = String::new(),
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

        let upower_proxy = UPowerProxy::builder(&conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let device_proxy = get_device_proxy(&conn, my_config.device_path).await?;

        for request in requests.iter_mut() {}

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, strum_macros::EnumDiscriminants)]
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
}

async fn get_device_proxy<'c>(
    conn: &'c Connection,
    device: String,
) -> zbus::Result<DeviceProxy<'c>> {
    let mut builder = DeviceProxy::builder(conn).cache_properties(CacheProperties::No);

    if !device.is_empty() {
        builder = builder.path(device)?;
    }

    builder.build().await
}
