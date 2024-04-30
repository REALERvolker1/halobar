mod xmlgen;

mod chosen;
mod variants;

use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;
use zbus::proxy::CacheProperties;

use self::{
    variants::{NMDeviceType, NMState},
    xmlgen::network_manager,
};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetData {
    pub ssid: Option<Arc<String>>,
    pub device: Option<Arc<String>>,
    pub up_speed: Option<u64>,
    pub down_speed: Option<u64>,
    pub state: NMState,
}

config_struct! {
    [NetIcon]
    asleep: char = '󰲚',
    connected_global: char = '󰱔',
    connected_local: char = '󰲁',
    connected_site: char = '󰲝',
    connecting: char = '󰲺',
    disconnected: char = '󰲜',
    disconnecting: char = '󰲝',
    unknown: char = '󰲊',
}

impl NetIconKnown {
    /// TODO: Icon config
    fn state_icon(&self, state: NMState) -> char {
        match state {
            NMState::Asleep => self.asleep,
            NMState::ConnectedGlobal => self.connected_global,
            NMState::ConnectedLocal => self.connected_local,
            NMState::ConnectedSite => self.connected_site,
            NMState::Connecting => self.connecting,
            NMState::Disconnected => self.disconnected,
            NMState::Disconnecting => self.disconnecting,
            NMState::Unknown => self.unknown,
        }
    }
    fn is_online(state: NMState) -> bool {
        match state {
            NMState::ConnectedGlobal | NMState::Unknown => true,
            _ => false,
        }
    }
}

// struct FormatNet {
//     data: NetData,
//     /// TODO: Allow choosing decimal rounding
//     format: FmtSegmentVec,
//     format_offline: FmtSegmentVec,
// }
// impl HaloFormatter for FormatNet {
//     type Data = NetData;
//     fn current_data<'a>(&'a self) -> &'a Self::Data {
//         &self.data
//     }
//     fn default_format_str() -> FormatStr {
//         "{icon} {up_speed} UP, {down_speed} DOWN".into()
//     }
//     fn fn_table<'a>(&'a self) -> halobar_config::fmt::FnTable<Self::Data, 1> {
//         FnTable([
//             ("icon", |data| Some(data.state.state_icon().to_string())),
//             ("up_speed", |data| Some(format!("{}", data.up_speed))),
//             ("down_speed", |data| Some(format!("{}", data.down_speed))),
//         ])
//     }
//     fn segments<'s>(&'s self) -> FmtSegments<'s> {
//         if self.data.
//     }
// }

config_struct! {
    [Net]
    interface: String = String::new(),
    show_speed_up: bool = true,
    show_speed_down: bool = true,
    show_ssid: bool = true,
    show_device: bool = true,
    show_state: bool = true,
    // format: FormatStr = FormatStr::default(),
}
impl NetKnown {
    pub fn is_valid(&self) -> bool {
        self.show_device
            && self.show_speed_down
            && self.show_speed_up
            && self.show_ssid
            && self.show_state
    }
}

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
    #[error("Failed to send network module channel to subscriber")]
    InitializerSendError,
}

pub struct Network {
    // /// The device in /sys/class/net
    // interface: Option<Arc<String>>,

    // connection: zbus::Connection,
    // channel: BiChannel<NetData, Event>,

    // show_speed_up: bool,
    // show_speed_down: bool,
    // show_ssid: bool,
    // show_device: bool,
    // show_state: bool,
}
impl Network {
    async fn init(
        config: NetKnown,
        conn: zbus::Connection,
        sender: oneshot::Sender<BiChannel<Event, NetData>>,
    ) -> Result<(), NetError> {
        if !config.is_valid() {
            return Err(NetError::NetDisabled);
        }

        let network_manager = network_manager::NetworkManagerProxy::builder(&conn)
            .build()
            .await?;

        macro_rules! init_channel {
            () => {
                let (channel, yours) = BiChannel::new(
                    5,
                    Some("Networkmanager module"),
                    Some("Networkmanager receiver"),
                );
                sender
                    .send(yours)
                    .map_err(|_| NetError::InitializerSendError)?;
            };
        }

        // if config.interface.is_empty() {
        //     let query = primary_connection(&network_manager, &conn).await;
        // }

        // let mut connection_query = {
        //     let query = {

        //     } else {
        //         chosen_connection(&config.interface, &network_manager, &conn).await
        //     };

        //     match query {
        //         Ok(t) => Some(t),
        //         Err(e) => match e {
        //             NetError::NetDisabled => None,
        //             _ => return Err(e),
        //         },
        //     }
        // };

        // if config.interface.is_empty() {} else {
        //     let mut devices_stream = network_manager.receive_devices_changed().await;

        //     while let Some(device) = devices_stream.next().await {
        //         let devices = device.get().await?;
        //         if devices.is_empty() || !devices.contains(x)

        //         connection_query = ;

        //         if let Some((active, device)) = connection_query {
        //         } else {
        //             let mut state_stream = network_manager.receive_state_changed().await;

        //             while let Some(state) = state_stream.next().await {
        //                 let current = state.get().await?;
        //                 debug!("Networkmanager state: {current}");
        //             }
        //         }
        //     }
        // }

        Ok(())
    }
}

/// When you chose the interface
async fn chosen_connection<'a>(
    interface_name: &str,
    network_manager: &network_manager::NetworkManagerProxy<'a>,
    conn: &'a zbus::Connection,
) -> Result<
    Option<(
        xmlgen::active_connection::ActiveProxy<'a>,
        xmlgen::device::DeviceProxy<'a>,
        zvariant::OwnedObjectPath,
    )>,
    NetError,
> {
    let mut devices = network_manager
        .devices()
        .await?
        .into_iter()
        .map(|d| async {
            let device_path = d.clone();
            let proxy = xmlgen::device::DeviceProxy::builder(&conn)
                .path(d)?
                .cache_properties(CacheProperties::No)
                .build()
                .await?;

            let iface = proxy.interface().await?;

            if iface == interface_name {
                return Ok((proxy, device_path));
            }

            Err(NetError::InvalidInterface(interface_name.to_owned()))
        })
        .collect::<FuturesUnordered<_>>();

    let mut device = None;

    while let Some(d) = devices.next().await {
        match d {
            Ok(dev) => {
                device.replace(dev);
                break;
            }
            Err(e) => {
                warn!("Networkmanager error: {e}");
            }
        }
    }

    let device = match device {
        Some(d) => d,
        None => return Err(NetError::InvalidInterface(interface_name.to_owned())),
    };

    let active_path = match device.0.active_connection().await {
        Ok(c) => c,
        Err(e) => {
            warn!("Error getting active connection for {interface_name}: {e}");
            return Ok(None);
        }
    };

    // This should not return errors.
    let active = xmlgen::active_connection::ActiveProxy::builder(conn)
        .path(active_path)?
        .build()
        .await?;

    Ok(Some((active, device.0, device.1)))
}

/// Autodetection
async fn primary_connection<'a>(
    network_manager: &network_manager::NetworkManagerProxy<'a>,
    conn: &'a zbus::Connection,
) -> Result<
    (
        xmlgen::active_connection::ActiveProxy<'a>,
        xmlgen::device::DeviceProxy<'a>,
    ),
    NetError,
> {
    let active_path = network_manager.primary_connection().await?;
    debug!("Active networkmanager connection: {active_path}");

    let active = xmlgen::active_connection::ActiveProxy::builder(conn)
        .path(active_path)?
        .build()
        .await?;
    let active_devices = active.devices().await?;

    let mut device = None;

    for path in active_devices {
        let proxy = xmlgen::device::DeviceProxy::builder(&conn)
            .path(path)?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let ty = proxy.device_type().await?;

        match ty {
            NMDeviceType::Dummy | NMDeviceType::Loopback => {
                trace!("Device has invalid network type, skipping...");
                continue;
            }
            _ => {
                device.replace(proxy);
                break;
            }
        }
    }

    // How do you have a connection active with no devices??
    let device = match device {
        Some(d) => d,
        None => unreachable!("No devices associated with active networkmanager connection! Please file a bug report!"),
    };

    Ok((active, device))
}

/// This is split into a different function so I can easily retry connecting.
async fn get_connected<'a>(
    network_manager: &network_manager::NetworkManagerProxy<'a>,
    conn: &'a zbus::Connection,
    interface: Option<&str>,
) -> Result<
    Option<(
        xmlgen::device::DeviceProxy<'a>,
        xmlgen::active_connection::ActiveProxy<'a>,
    )>,
    NetError,
> {
    let active_devices = network_manager.devices().await?;
    if active_devices.is_empty() {
        return Ok(None);
    }
    let mut device_proxy = None;

    for device in active_devices.into_iter() {
        let proxy = xmlgen::device::DeviceProxy::builder(&conn)
            .path(device)?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        match interface.as_ref() {
            Some(name) => {
                let iface = match proxy.interface().await {
                    Ok(i) => i,
                    Err(e) => {
                        warn!("Error getting proxy interface: {e}");
                        continue;
                    }
                };

                if name == &iface {
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
                interface.unwrap_or("None").to_owned(),
            ))
        }
    };

    // I got this from the list of active devices. It should just work.
    let active_connection_path = match device.active_connection().await {
        Ok(c) => c,
        Err(e) => {
            warn!("Could not get the active networkmanager connection: {e}");
            return Ok(None);
        }
    };
    let active_connection = xmlgen::active_connection::ActiveProxy::builder(&conn)
        .path(active_connection_path)?
        .build()
        .await?;

    Ok(Some((device, active_connection)))
}
