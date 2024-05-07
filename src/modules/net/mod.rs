use std::{ffi::OsString, os::unix::ffi::OsStringExt};

use neli_wifi::{Bss, Station};

use super::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct InterfaceStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}
impl std::ops::Sub for InterfaceStats {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.rx_bytes = self.rx_bytes.saturating_sub(rhs.rx_bytes);
        self.tx_bytes = self.tx_bytes.saturating_sub(rhs.tx_bytes);
        self
    }
}

// From `linux/rtnetlink.h`
// const RT_SCOPE_HOST: c_uchar = 254;

// #[derive(Debug)]
// pub struct NetDevice {
//     pub iface: Interface,
//     pub wifi_info: Option<WifiInfo>,
//     pub ip: Option<Ipv4Addr>,
//     pub ipv6: Option<Ipv6Addr>,
//     pub icon: &'static str,
//     pub tun_wg_ppp: bool,
// }

#[derive(Debug, Default)]
pub struct WifiInfo {
    pub ssid: Option<String>,
    pub signal: Option<f64>,
    pub frequency: Option<f64>,
    pub bitrate: Option<f64>,
}

/// https://github.com/greshake/i3status-rust/blob/fc5a3f69a1b7cfc1fcb636ea05a46f08b7f4b095/src/netlink.rs#L431
///
/// Original Source: https://www.kernel.org/doc/Documentation/networking/operstates.txt
#[derive(Debug, PartialEq, Eq)]
pub enum Operstate {
    /// Interface is in unknown state, neither driver nor userspace has set
    /// operational state. Interface must be considered for user data as
    /// setting operational state has not been implemented in every driver.
    Unknown,
    /// Unused in current kernel (notpresent interfaces normally disappear),
    /// just a numerical placeholder.
    Notpresent,
    /// Interface is unable to transfer data on L1, f.e. ethernet is not
    /// plugged or interface is ADMIN down.
    Down,
    /// Interfaces stacked on an interface that is IF_OPER_DOWN show this
    /// state (f.e. VLAN).
    Lowerlayerdown,
    /// Unused in current kernel.
    Testing,
    /// Interface is L1 up, but waiting for an external event, f.e. for a
    /// protocol to establish. (802.1X)
    Dormant,
    /// Interface is operational up and can be used.
    Up,
}
impl From<u8> for Operstate {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Notpresent,
            2 => Self::Down,
            3 => Self::Lowerlayerdown,
            4 => Self::Testing,
            5 => Self::Dormant,
            6 => Self::Up,
            _ => Self::Unknown,
        }
    }
}

pub type NeliDeviceIndex = i32;

#[derive(Debug, Default, derive_more::Display)]
#[display(fmt = "{}. {}: {}", device_index, name, ssid)]
pub struct WifiInterface {
    /// Used internally by neli
    pub index: NeliDeviceIndex,
    pub ssid: String,
    pub name: String,
    pub power: u32,
    // pub rx_packets: u64,
    // pub tx_packets: u64,
}
impl WifiInterface {
    pub fn from_neli_wifi(interface: neli_wifi::Interface) -> NetResult<Self> {
        let me = Self {
            index: interface
                .index
                .ok_or_else(|| NetError::MissingProperty("index"))?,
            ssid: denullify(
                interface
                    .ssid
                    .ok_or_else(|| NetError::MissingProperty("ssid"))?,
            )
            .unwrap_or_default(),
            name: denullify(
                interface
                    .name
                    .ok_or_else(|| NetError::MissingProperty("name"))?,
            )
            .unwrap_or_default(),
            power: interface
                .power
                .ok_or_else(|| NetError::MissingProperty("power"))?,
        };

        Ok(me)
    }
    #[instrument(level = "trace", skip(socket))]
    async fn from_name(socket: &mut neli_wifi::AsyncSocket, device_name: &str) -> NetResult<Self> {
        let mut matching_interfaces = socket
            .get_interfaces_info()
            .await?
            .into_iter()
            .filter_map(|mut iface| {
                // I need to replace with an empty vec or I will trigger a missing property error
                let name = iface.name.replace(Vec::new())?;
                let name = denullify(name)?;

                if name == device_name {
                    return Some(Self::from_neli_wifi(iface));
                }

                None
            })
            .filter_map(|iface| {
                match iface {
                    Ok(i) => return Some(i),
                    Err(e) => warn!("Interface error: {e}"),
                }

                None
            });

        // I only check for one because there should only be one matching interface.
        let Some(interface) = matching_interfaces.next() else {
            return Err(NetError::InvalidIface(device_name.to_owned()));
        };

        // This allows it to just ignore duplicates instead of failing outright, while telling you it did so
        while let Some(iface) = matching_interfaces.next() {
            warn!("Duplicate interface: {}", iface);
        }

        Ok(interface)
    }
    async fn from_index(
        socket: &mut neli_wifi::AsyncSocket,
        index: NeliDeviceIndex,
    ) -> NetResult<Self> {
        let mut matching_iface =
            socket
                .get_interfaces_info()
                .await?
                .into_iter()
                .filter_map(|iface| {
                    let idx = iface.index?;
                    if idx == index {
                        return Some(iface);
                    }

                    None
                });

        let matching = match matching_iface.next() {
            Some(i) => i,
            None => return Err(NetError::InvalidIndex(Some(index))),
        };

        let device = Self::from_neli_wifi(matching)?;

        Ok(device)
    }
}

// #[derive(Debug, Default)]
// pub struct WifiInterface {
//     interface: WifiInterfaceMinimal,
//     stations: Vec<Station>,
//     bss: Vec<Bss>,
// }
// impl WifiInterface {
//     pub async fn query(socket: &mut neli_wifi::AsyncSocket) -> NetResult<Vec<Self>> {
//         let interfaces = socket
//             .get_interfaces_info()
//             .await?
//             .into_iter()
//             .filter_map(WifiInterfaceMinimal::from_neli_wifi);

//         // it all requires a &mut borrow so I can't join these futures ughhhh
//         let mut out = Vec::new();

//         for interface in interfaces {
//             debug!("Getting neli device information");
//             let stations = match socket.get_station_info(interface.device_index).await {
//                 Ok(s) => s,
//                 Err(e) => {
//                     warn!("Failed getting wifi stations: {e}");
//                     continue;
//                 }
//             };

//             let bss = match socket.get_bss_info(interface.device_index).await {
//                 Ok(b) => b,
//                 Err(e) => {
//                     warn!("Failed to get wifi BSS info: {e}");
//                     continue;
//                 }
//             };

//             out.push(WifiInterface {
//                 interface,
//                 stations,
//                 bss,
//             })
//         }

//         Ok(out)
//     }
// }

/// A function to fix strings because I was getting nullbytes at the end of some of them
fn denullify(mut bytes: Vec<u8>) -> Option<String> {
    let last_byte = bytes.pop()?;
    if last_byte != 0 {
        bytes.push(last_byte);
    }

    Some(String::from_utf8_lossy(&bytes).into_owned())
}

pub async fn run() -> R<()> {
    let mut sock = neli_wifi::AsyncSocket::connect()?;
    let interfaces = WifiInterface::query(&mut sock).await?;

    for iface in interfaces {
        let bss = iface.bss.first().unwrap().clone();
        let station = iface.stations.first().unwrap().clone();

        let info = station.rx_bitrate.unwrap();

        info!("{:?}", iface);
        info!("{}", info);
    }

    Ok(())
}

/// An error that can occur in the net module
#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("Invalid index: {:?}", 0)]
    InvalidIndex(Option<NeliDeviceIndex>),
    #[error("Invalid interface: {0}")]
    InvalidIface(String),
    #[error("Missing required property: {0}")]
    MissingProperty(&'static str),
    #[error("{0}")]
    Neli(String),
    #[error("Internal error: {0}")]
    Internal(&'static str),
}
impl From<neli::err::NlError> for NetError {
    fn from(e: neli::err::NlError) -> Self {
        Self::Neli(e.to_string())
    }
}

pub type NeliResult<T> = Result<T, neli::err::NlError>;

pub type NetResult<T> = std::result::Result<T, NetError>;
