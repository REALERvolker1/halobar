use std::{ffi::OsString, os::unix::ffi::OsStringExt};

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

#[derive(Debug, Default)]
pub struct WifiInterface {
    /// Used internally by neli
    pub device_index: NeliDeviceIndex,
    pub ssid: String,
    pub name: String,
    pub power: u32,
}
impl WifiInterface {
    pub fn from_neli_wifi(interface: neli_wifi::Interface) -> Option<Self> {
        Some(Self {
            device_index: interface.index?,
            ssid: denullify(interface.ssid?)?,
            name: denullify(interface.name?)?,
            power: interface.power?,
        })
    }
}

/// A function to fix strings because I was getting nullbytes at the end of some of them
fn denullify(mut bytes: Vec<u8>) -> Option<String> {
    let last_byte = bytes.pop()?;
    if last_byte != 0 {
        bytes.push(last_byte);
    }

    OsString::from_vec(bytes).into_string().ok()
}

pub async fn run() -> R<()> {
    let mut sock = neli_wifi::AsyncSocket::connect()?;
    let interfaces = sock
        .get_interfaces_info()
        .await?
        .into_iter()
        .filter_map(WifiInterface::from_neli_wifi);

    for iface in interfaces {
        let info = sock.get_station_info(iface.device_index).await?;
        info!("{:?}", iface);
        warn!("{:?}", info);
    }

    Ok(())
}

/// An error that can occur in the net module
#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("Invalid index: {:?}", 0)]
    InvalidIndex(Option<NeliDeviceIndex>),
    #[error("{0}")]
    Neli(#[from] neli::err::WrappedError),
}

pub type NetResult<T> = std::result::Result<T, NetError>;
