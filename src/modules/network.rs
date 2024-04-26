mod access_point;
mod active_connection;
mod device;
mod network_manager;
mod settings;

use zbus::proxy::CacheProperties;

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
    network_manager: network_manager::NetworkManagerProxy<'c>,
    settings: settings::SettingsProxy<'c>,
    device: device::DeviceProxy<'c>,
    stats: device::StatisticsProxy<'c>,
    active: active_connection::ActiveProxy<'c>,
    access_point: access_point::AccessPointProxy<'c>,
}
impl<'c> Proxies<'c> {
    pub async fn new(
        conn: &'c zbus::Connection,
        iface_name: Option<&str>,
    ) -> Result<Proxies<'c>, NetError> {
        // TODO: Make this a function input
        let nm_proxy = network_manager::NetworkManagerProxy::builder(conn)
            .build()
            .await?;

        let is_enabled = nm_proxy.connectivity_check_enabled().await?;
        if !is_enabled {
            return Err(NetError::NetDisabled);
        }

        // let active_connections = nm_proxy.active_connections().await?;

        let active_devices = nm_proxy.devices().await?;
        if active_devices.is_empty() {
            return Err(NetError::NetDisabled);
        }
        let mut device_proxy = None;

        for device in active_devices {
            let proxy = device::DeviceProxy::builder(conn)
                .path(device)?
                .cache_properties(CacheProperties::No)
                .build()
                .await?;

            match iface_name {
                Some(name) => {
                    let iface = proxy.interface().await?;
                    if iface == name {
                        device_proxy.replace(proxy);
                    }
                }
                None => {
                    proxy.
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

        return Err(NetError::NetDisabled);
    }
}

macro_rules! owned_repr {
    ($ty:ty) => {
        impl TryFrom<zvariant::OwnedValue> for $ty {
            type Error = zvariant::Error;
            fn try_from(value: zvariant::OwnedValue) -> Result<Self, Self::Error> {
                match value.downcast_ref().map(Self::from_repr) {
                    Ok(Some(v)) => Ok(v),
                    _ => Err(zvariant::Error::IncorrectType),
                }
            }
        }
    };
}

/// The [NMConnectivityState](https://networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMConnectivityState) type from networkmanager
///
/// Documentation ported over verbatim
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    zvariant::Type,
    Deserialize_repr,
    Serialize_repr,
    strum_macros::FromRepr,
)]
#[repr(u32)]
pub enum NMConnectivityState {
    /// Network connectivity is unknown.
    /// This means the connectivity checks are disabled (e.g. on server installations) or has not run yet.
    ///
    /// The graphical shell should assume the Internet connection might be available and not present a captive portal window.
    #[default]
    Unknown = 0,
    /// The host is not connected to any network.
    /// There's no active connection that contains a default route to the internet and thus it makes no sense to even attempt a connectivity check.
    ///
    /// The graphical shell should use this state to indicate the network connection is unavailable.
    None = 1,
    /// The Internet connection is hijacked by a captive portal gateway.
    ///
    /// The graphical shell may open a sandboxed web browser window
    /// (because the captive portals typically attempt a man-in-the-middle attacks against the https connections)
    /// for the purpose of authenticating to a gateway and retrigger the connectivity check with CheckConnectivity() when the browser window is dismissed.
    Portal = 2,
    /// The host is connected to a network, does not appear to be able to reach the full Internet, but a captive portal has not been detected.
    Limited = 3,
    /// The host is connected to a network, and appears to be able to reach the full Internet.
    Full = 4,
}
owned_repr!(NMConnectivityState);

/// [NMState](https://networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMState) values indicate the current overall networking state.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    zvariant::Type,
    Deserialize_repr,
    Serialize_repr,
    strum_macros::FromRepr,
)]
#[repr(u32)]
pub enum NMState {
    /// Networking state is unknown.
    /// This indicates a daemon error that makes it unable to reasonably assess the state.
    ///
    /// In such event the applications are expected to assume Internet connectivity might be present and not disable controls that require network access.
    /// The graphical shells may hide the network accessibility indicator altogether since no meaningful status indication can be provided.
    #[default]
    Unknown = 0,
    /// Networking is not enabled, the system is being suspended or resumed from suspend.
    Asleep = 10,
    /// There is no active network connection.
    ///
    /// The graphical shell should indicate no network connectivity and the applications should not attempt to access the network.
    Disconnected = 20,
    /// Network connections are being cleaned up. The applications should tear down their network sessions.
    Disconnecting = 30,
    /// A network connection is being started.
    ///
    /// The graphical shell should indicate the network is being connected while the applications should still make no attempts to connect the network.
    Connecting = 40,
    /// There is only local IPv4 and/or IPv6 connectivity, but no default route to access the Internet.
    ///
    /// The graphical shell should indicate no network connectivity.
    ConnectedLocal = 50,
    /// There is only site-wide IPv4 and/or IPv6 connectivity.
    /// This means a default route is available, but the Internet connectivity check (see "Connectivity" property) did not succeed.
    ///
    /// The graphical shell should indicate limited network connectivity.
    ConnectedSite = 60,
    /// There is global IPv4 and/or IPv6 Internet connectivity.
    ///
    /// This means the Internet connectivity check succeeded, the graphical shell should indicate full network connectivity.
    ConnectedGlobal = 70,
}

/// [NMDeviceType](https://networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMDeviceType) values indicate the type of hardware represented by a device object.
#[allow(non_camel_case_types)]
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    zvariant::Type,
    Deserialize_repr,
    Serialize_repr,
    strum_macros::FromRepr,
    strum_macros::Display,
)]
#[repr(u32)]
pub enum NMDeviceType {
    /// unknown device
    #[default]
    Unknown = 0,
    /// generic support for unrecognized device types
    Generic = 14,
    /// a wired ethernet device
    Ethernet = 1,
    /// an 802.11 Wi-Fi device
    Wifi = 2,
    /// not used
    Unused1 = 3,
    /// not used
    Unused2 = 4,
    /// a Bluetooth device supporting PAN or DUN access protocols
    Bt = 5,
    /// an OLPC XO mesh networking device
    OLPC_Mesh = 6,
    /// an 802.16e Mobile WiMAX broadband device
    WIMAX = 7,
    /// a modem supporting analog telephone, CDMA/EVDO, GSM/UMTS, or LTE network access protocols
    Modem = 8,
    /// an IP-over-InfiniBand device
    InfiniBand = 9,
    /// a bond master interface
    Bond = 10,
    /// an 802.1Q VLAN interface
    Vlan = 11,
    /// ADSL modem
    ADSL = 12,
    /// a bridge master interface
    Bridge = 13,
    /// a team master interface
    Team = 15,
    /// a TUN or TAP interface
    TUN = 16,
    /// a IP tunnel interface
    IP_Tunnel = 17,
    /// a MACVLAN interface
    MACVLAN = 18,
    /// a VXLAN interface
    VXLAN = 19,
    /// a VETH interface
    VETH = 20,
    /// a MACsec interface
    MACsec = 21,
    /// a dummy interface
    Dummy = 22,
    /// a PPP interface
    PPP = 23,
    /// a Open vSwitch interface
    OVS_Interface = 24,
    /// a Open vSwitch port
    OVS_Port = 25,
    /// a Open vSwitch bridge
    OVS_Bridge = 26,
    /// a IEEE 802.15.4 (WPAN) MAC Layer Device
    WPAN = 27,
    /// 6LoWPAN interface
    LoWPAN = 28,
    /// a WireGuard interface
    Wireguard = 29,
    /// an 802.11 Wi-Fi P2P device. Since: 1.16.
    WIFI_P2P = 30,
    /// A VRF (Virtual Routing and Forwarding) interface. Since: 1.24.
    VRF = 31,
    /// a loopback interface. Since: 1.42.
    Loopback = 32,
    /// A HSR/PRP device. Since: 1.46.
    HSR = 33,
}
owned_repr!(NMDeviceType);
