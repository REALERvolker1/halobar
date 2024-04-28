mod xmlgen;

mod variants;

use zbus::proxy::CacheProperties;

use self::variants::NMDeviceType;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetData {
    // TODO: Figure out how to get this
    // pub ssid: Arc<String>,
    pub up_speed: u64,
    pub down_speed: u64,
    pub is_online: bool,
}

struct FormatNet {
    data: NetData,
    /// TODO: Allow choosing decimal rounding
    format: FmtSegmentVec,
    format_offline: FmtSegmentVec,
}
// impl HaloFormatter for FormatNet {
//     type Data = NetData;
//     fn current_data<'a>(&'a self) -> &'a Self::Data {
//         &self.data
//     }
//     fn default_format_str() -> FormatStr {
//         ""
//     }
// }

// config_struct! {
//     [Net]
//     format: FormatStr = FormatStr::default(),
// }

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("Invalid interface: {0}")]
    InvalidInterface(String),
    #[error("Nix errno: {0}")]
    Errno(#[from] Errno),
    #[error("{0}")]
    Io(#[from] tokio::io::Error),
    #[error("Error parsing integer: {0}")]
    Parse(#[from] std::num::ParseIntError),
    #[error("zbus error: {0}")]
    Zbus(#[from] zbus::Error),
    #[error("Networking disabled")]
    NetDisabled,
}

pub struct Network {
    /// The device in /sys/class/net
    interface: Arc<String>,

    last_data: NetData,
    connection: zbus::Connection,
    channel: BiChannel<String, Event>,
}
impl Network {
    // fn refresh(&mut self) -> Result<(), NetError> {
    //     // I get that doing it in this order is a bit more innacurate, but I would rather overestimate than underestimate in this instance.
    //     let last_checked = Instant::now();
    //     let since_last = last_checked.duration_since(self.last_checked);
    //     self.last_checked = last_checked;
    //     let seconds = since_last.as_secs();

    //     let data = NetData {
    //         up_speed: Self::speed_difference(seconds, self.last_data.up_speed, &self.tx_packets)?,
    //         down_speed: Self::speed_difference(
    //             seconds,
    //             self.last_data.down_speed,
    //             &self.rx_packets,
    //         )?,
    //     };

    //     Ok(())
    // }
    // /// Quick and dirty way to query one of the tx or rx things
    // fn speed_difference(time_seconds: u64, previous: u64, path: &Path) -> Result<u64, NetError> {
    //     let current = fs::read_to_string(path)?.parse::<u64>()?;
    //     let difference = current.saturating_sub(previous);

    //     let size_bytes = difference / time_seconds;
    //     Ok(size_bytes)
    // }
}
// impl BackendModule for Network

pub struct Proxies<'c> {
    network_manager: xmlgen::network_manager::NetworkManagerProxy<'c>,
    settings: xmlgen::settings::SettingsProxy<'c>,
    device: xmlgen::device::DeviceProxy<'c>,
    stats: xmlgen::device::StatisticsProxy<'c>,
    active: xmlgen::active_connection::ActiveProxy<'c>,
    access_point: xmlgen::access_point::AccessPointProxy<'c>,
}
impl<'c> Proxies<'c> {
    pub async fn new(
        conn: &'c zbus::Connection,
        iface_name: Option<&str>,
    ) -> Result<Proxies<'c>, NetError> {
        // TODO: Make this a function input
        let network_manager = xmlgen::network_manager::NetworkManagerProxy::builder(conn)
            .build()
            .await?;

        let is_enabled = network_manager.connectivity_check_enabled().await?;
        if !is_enabled {
            return Err(NetError::NetDisabled);
        }

        // let active_connections = network_manager.active_connections().await?;

        let active_devices = network_manager.devices().await?;
        if active_devices.is_empty() {
            return Err(NetError::NetDisabled);
        }
        let mut device_proxy = None;

        for device in active_devices {
            let proxy = xmlgen::device::DeviceProxy::builder(conn)
                .path(device)?
                .cache_properties(CacheProperties::No)
                .build()
                .await?;

            match iface_name {
                Some(name) => {
                    let iface = match proxy.interface().await {
                        Ok(i) => i,
                        Err(e) => {
                            warn!("Error getting proxy interface: {e}");
                            continue;
                        }
                    };

                    if iface == name {
                        device_proxy.replace(proxy);
                        break;
                    }
                }
                None => {
                    let device_type = match proxy.device_type().await {
                        Ok(d) => d,
                        Err(e) => {
                            warn!("Error getting proxy device type: {e}");
                            continue;
                        }
                    };

                    // This is very naive. the user should ideally have a wifi or ethernet connection in the
                    // majority of cases, I can't think of a better way to do this without bloating things to hell.
                    match device_type {
                        NMDeviceType::Ethernet | NMDeviceType::Wifi | NMDeviceType::WIFI_P2P => {
                            device_proxy.replace(proxy);
                            break;
                        }
                        _ => {
                            warn!("Invalid device type: {device_type}, skipping");
                        }
                    }
                }
            }
        }

        let device = match device_proxy {
            Some(d) => d,
            None => {
                return Err(NetError::InvalidInterface(
                    iface_name.unwrap_or("None").to_owned(),
                ))
            }
        };

        // I got this from the list of active devices. It should just work.
        let active_connection = device.active_connection().await?;
        let active = xmlgen::active_connection::ActiveProxy::builder(conn)
            .path(active_connection)?
            .build()
            .await?;

        // let proxies = Proxies {
        //     network_manager,
        //     device,
        //     active,
        // };

        return Err(NetError::NetDisabled);
    }
}
