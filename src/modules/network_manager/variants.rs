//! A lot of this documentation was lifted from [networkmanager.dev].
//!
//! TODO: The enums with values of 0x.... are supposed to be merged sort of like drwxr--r-- signs on folders

use crate::prelude::{zvariant, Deserialize, Deserialize_repr, Serialize, Serialize_repr};

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
        impl TryFrom<zvariant::Value<'_>> for $ty {
            type Error = zvariant::Error;
            fn try_from(value: zvariant::Value<'_>) -> Result<Self, Self::Error> {
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
    strum_macros::Display,
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
owned_repr!(NMState);

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

// /// General device capability flags.
// #[derive(
//     Debug,
//     Default,
//     Clone,
//     Copy,
//     PartialEq,
//     Eq,
//     PartialOrd,
//     Ord,
//     zvariant::Type,
//     Deserialize_repr,
//     Serialize_repr,
//     strum_macros::FromRepr,
//     strum_macros::Display,
// )]
// #[repr(u32)]
// pub enum NMDeviceCapabilities {
//     /// device has no special capabilities
//     #[default]
//     None = 0x00000000,
//     /// NetworkManager supports this device
//     NMSupported = 0x00000001,
//     /// this device can indicate carrier status
//     CarrierDetect = 0x00000002,
//     /// this device is a software device
//     IsSoftware = 0x00000004,
//     /// this device supports single-root I/O virtualization
//     SRIOV = 0x00000008,
// }
// owned_repr!(NMDeviceCapabilities);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NMDeviceCapabilities(pub u32);

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct NMDeviceCapabilitiesFlags: u32 {
        const NONE = 0x00000000;
        const NM_SUPPORTED = 0x00000001;
        const CARRIER_DETECT = 0x00000002;
        const IS_SOFTWARE = 0x00000004;
        const SRIOV = 0x00000008;
    }
}

/// [NMCapability](https://networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMCapability) names the numbers in the Capabilities property.
/// Capabilities are positive numbers. They are part of stable API and a certain capability number is guaranteed not to change.
///
/// The range 0x7000 - 0x7FFF of capabilities is guaranteed not to be used by upstream NetworkManager.
/// It could thus be used for downstream extensions.
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
pub enum NMCapability {
    #[default]
    Team = 1,
    OVS = 2,
}
owned_repr!(NMCapability);

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    zvariant::Type,
    Deserialize,
    Serialize,
    derive_more::AsRef,
    derive_more::Deref,
    derive_more::IntoIterator,
    derive_more::From,
)]
pub struct NMCapabilityVec(pub Vec<NMCapability>);
impl TryFrom<zvariant::OwnedValue> for NMCapabilityVec {
    type Error = zvariant::Error;
    fn try_from(value: zvariant::OwnedValue) -> Result<Self, Self::Error> {
        if let zvariant::Value::Array(arr) = value.downcast_ref()? {
            // I know this is unnecessarily expensive, but I intend to only call this in one specific place and it should theoretically be infallible
            let mut into = Vec::with_capacity(arr.len());

            for item in arr.into_iter() {
                match item {
                    zvariant::Value::U32(u) => match NMCapability::from_repr(*u) {
                        Some(s) => into.push(s),
                        None => return Err(zvariant::Error::IncorrectType),
                    },
                    _ => return Err(zvariant::Error::IncorrectType),
                }
            }

            return Ok(NMCapabilityVec(into));
        }

        Err(zvariant::Error::IncorrectType)
    }
}

/// Indicates the 802.11 mode an access point or device is currently in.
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
pub enum NM80211Mode {
    /// the device or access point mode is unknown
    #[default]
    Unknown = 0,
    /// for both devices and access point objects, indicates the object is part of an Ad-Hoc 802.11 network without a central coordinating access point.
    AdHoc = 1,
    /// the device or access point is in infrastructure mode.
    /// - For devices, this indicates the device is an 802.11 client/station.
    /// - For access point objects, this indicates the object is an access point that provides connectivity to clients.
    Infra = 2,
    /// the device is an access point/hotspot. Not valid for access point objects; used only for hotspot mode on the local machine.
    AP = 3,
    /// the device is a 802.11s mesh point. Since: 1.20.
    Mesh = 4,
}
owned_repr!(NM80211Mode);

/// The device state
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
pub enum NMDeviceState {
    /// the device's state is unknown
    #[default]
    Unknown = 0,
    /// the device is recognized, but not managed by NetworkManager
    Unmanaged = 10,
    /// the device is managed by NetworkManager, but is not available for use.
    ///
    /// Reasons may include the wireless switched off, missing firmware, no ethernet carrier, missing supplicant or modem manager, etc.
    Unavailable = 20,
    /// the device can be activated, but is currently idle and not connected to a network.
    Disconnected = 30,
    /// the device is preparing the connection to the network.
    ///
    /// This may include operations like changing the MAC address, setting physical link properties, and anything else required to connect to the requested network.
    Prepare = 40,
    /// the device is connecting to the requested network.
    ///
    /// This may include operations like associating with the Wi-Fi AP, dialing the modem, connecting to the remote Bluetooth device, etc.
    Config = 50,
    /// the device requires more information to continue connecting to the requested network.
    ///
    /// This includes secrets like WiFi passphrases, login passwords, PIN codes, etc.
    NeedAuth = 60,
    /// the device is requesting IPv4 and/or IPv6 addresses and routing information from the network.
    IpConfig = 70,
    /// the device is checking whether further action is required for the requested network connection.
    ///
    /// This may include checking whether only local network access is available, whether a captive portal is blocking access to the Internet, etc.
    IpCheck = 80,
    /// the device is waiting for a secondary connection (like a VPN) which must activated before the device can be activated
    Secondaries = 90,
    /// the device has a network connection, either local or global.
    Activated = 100,
    /// a disconnection from the current network connection was requested, and the device is cleaning up resources used for that connection.
    ///
    /// The network connection may still be valid.
    Deactivating = 110,
    /// the device failed to connect to the requested network and is cleaning up the connection request
    Failed = 120,
}
owned_repr!(NMDeviceState);

/// The result of a checkpoint Rollback() operation for a specific device.
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
pub enum NMRollbackResult {
    /// the rollback succeeded.
    Ok = 0,
    /// the device no longer exists.
    NoDevice = 1,
    /// the device is now unmanaged.
    DeviceUnmanaged = 2,
    /// other errors during rollback.
    #[default]
    Failed = 3,
}
owned_repr!(NMRollbackResult);

/// Flags describing the current activation state.
///
/// TODO: Make a bitflags!
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
pub enum NMSettingsConnectionFlags {
    /// an alias for numeric zero, no flags set.
    #[default]
    None = 0,
    /// the connection is not saved to disk.
    ///
    /// That either means, that the connection is in-memory only and currently is not backed by a file.
    /// Or, that the connection is backed by a file, but has modifications in-memory that were not persisted to disk.
    Unsaved = 0x01,
    /// A connection is "nm-generated" if it was generated by NetworkManger.
    ///
    /// If the connection gets modified or saved by the user, the flag gets cleared.
    /// A nm-generated is also unsaved and has no backing file as it is in-memory only.
    NmGenerated = 0x02,
    /// The connection will be deleted when it disconnects.
    ///
    /// That is for in-memory connections (unsaved), which are currently active but deleted on disconnect.
    /// Volatile connections are always unsaved, but they are also no backing file on disk and are entirely in-memory only.
    Volatile = 0x04,
    /// the profile was generated to represent an external configuration of a networking device. Since: 1.26.
    External = 0x08,
}
owned_repr!(NMSettingsConnectionFlags);

/// The NMMetered enum has two different purposes:
/// - one is to configure "connection.metered" setting of a connection profile in NMSettingConnection
/// - the other is to express the actual metered state of the NMDevice at a given moment.
///
/// For the connection profile only NM_METERED_UNKNOWN, NM_METERED_NO and NM_METERED_YES are allowed.
///
/// The device's metered state at runtime is determined by the profile which is currently active.
/// If the profile explicitly specifies NM_METERED_NO or NM_METERED_YES, then the device's metered state is as such.
/// If the connection profile leaves it undecided at NM_METERED_UNKNOWN (the default), then NetworkManager tries to guess the metered state,
/// for example based on the device type or on DHCP options (like Android devices exposing a "ANDROID_METERED" DHCP vendor option).
/// This then leads to either NM_METERED_GUESS_NO or NM_METERED_GUESS_YES.
///
/// Most applications probably should treat the runtime state NM_METERED_GUESS_YES like NM_METERED_YES, and all other states as not metered.
///
/// Note that the per-device metered states are then combined to a global metered state.
/// This is basically the metered state of the device with the best default route.
/// However, that generalization of a global metered state may not be correct if the default routes for IPv4 and IPv6 are on different devices, or if policy routing is configured.
/// In general, the global metered state tries to express whether the traffic is likely metered, but since that depends on the traffic itself, there is not one answer in all cases.
/// Hence, an application may want to consider the per-device's metered states.
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
pub enum NMMetered {
    /// The metered status is unknown
    #[default]
    Unknown = 0,
    /// Metered, the value was explicitly configured
    Yes = 1,
    /// Not metered, the value was explicitly configured
    No = 2,
    /// Metered, the value was guessed
    GuessYes = 3,
    /// Not metered, the value was guessed
    GuessNo = 4,
}
owned_repr!(NMMetered);

/// Flags related to radio interfaces.
///
/// TODO: Make a bitflags
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
pub enum NMRadioFlags {
    /// an alias for numeric zero, no flags set.
    #[default]
    None = 0,
    /// A Wireless LAN device or rfkill switch is detected in the system.
    WLANAvailable = 0x1,
    /// A Wireless WAN device or rfkill switch is detected in the system.
    WWANAvailable = 0x2,
}
owned_repr!(NMRadioFlags);

/// `%_NM_VERSION_INFO_CAPABILITY_UNUSED`: a dummy capability. It has no meaning, don't use it.
///
/// Currently no enum values are defined. These capabilities are exposed on D-Bus in the "VersionInfo" bit field.
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
pub enum NMVersionInfoCapability {
    /// rust made me put something here
    #[default]
    None,
}
owned_repr!(NMVersionInfoCapability);

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    zvariant::Type,
    Deserialize,
    Serialize,
    derive_more::AsRef,
    derive_more::Deref,
    derive_more::Display,
    derive_more::From,
)]
pub struct Ssid(pub String);
impl TryFrom<zvariant::OwnedValue> for Ssid {
    type Error = zvariant::Error;
    fn try_from(value: zvariant::OwnedValue) -> Result<Self, Self::Error> {
        if let zvariant::Value::Array(a) = value.downcast_ref()? {
            // This should be infallible as well and should not drop characters where I put it
            let collected = a
                .into_iter()
                .filter_map(|v| v.try_into().ok())
                .collect::<Vec<u8>>();

            if let Ok(s) = String::from_utf8(collected) {
                return Ok(Self(s));
            }
        }

        Err(zvariant::Error::IncorrectType)
    }
}

/// 802.11 access point flags.
///
/// TODO: Make a bitflags
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
pub enum NM80211ApFlags {
    /// access point has no special capabilities
    #[default]
    None = 0x00000000,
    /// access point requires authentication and encryption (usually means WEP)
    Privacy = 0x00000001,
    /// access point supports some WPS method
    WPS = 0x00000002,
    /// access point supports push-button WPS
    WPSPushButton = 0x00000004,
    /// access point supports PIN-based WPS
    WPSPin = 0x00000008,
}
owned_repr!(NM80211ApFlags);

/// 802.11 access point security and authentication flags.
///
/// These flags describe the current security requirements of an access point as determined from the access point's beacon.
///
/// TODO: Make a bitflags
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
pub enum NM80211ApSecurityFlags {
    /// the access point has no special security requirements
    #[default]
    None = 0x00000000,
    /// 40/64-bit WEP is supported for pairwise/unicast encryption
    PairWEP40 = 0x00000001,
    /// 104/128-bit WEP is supported for pairwise/unicast encryption
    PairWEP104 = 0x00000002,
    /// TKIP is supported for pairwise/unicast encryption
    PairTKIP = 0x00000004,
    /// AES/CCMP is supported for pairwise/unicast encryption
    PairCCMP = 0x00000008,
    /// 40/64-bit WEP is supported for group/broadcast encryption
    GroupWEP40 = 0x00000010,
    /// 104/128-bit WEP is supported for group/broadcast encryption
    GroupWEP104 = 0x00000020,
    /// TKIP is supported for group/broadcast encryption
    GroupTKIP = 0x00000040,
    /// AES/CCMP is supported for group/broadcast encryption
    GroupCCMP = 0x00000080,
    /// WPA/RSN Pre-Shared Key encryption is supported
    KeyMgmtPSK = 0x00000100,
    /// 802.1x authentication and key management is supported
    KeyMgmt8021x = 0x00000200,
    /// WPA/RSN Simultaneous Authentication of Equals is supported
    KeyMgmtSAE = 0x00000400,
    /// WPA/RSN Opportunistic Wireless Encryption is supported
    KeyMgmtOWE = 0x00000800,
    /// WPA/RSN Opportunistic Wireless Encryption transition mode is supported. Since: 1.26.
    KeyMgmtOWETransitionMode = 0x00001000,
    /// WPA3 Enterprise Suite-B 192 bit mode is supported. Since: 1.30.
    KeyMgmtEAP = 0x00002000,
}
owned_repr!(NM80211ApSecurityFlags);

/// [NMActiveConnectionState](https://networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMActiveConnectionState)
/// values indicate the state of a connection to a specific network while it is starting, connected, or disconnecting from that network.
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
pub enum NMActiveConnectionState {
    /// the state of the connection is unknown
    #[default]
    Unknown = 0,
    /// a network connection is being prepared
    Activating = 1,
    /// there is a connection to the network
    Activated = 2,
    /// the network connection is being torn down and cleaned up
    Deactivating = 3,
    /// the network connection is disconnected and will be removed
    Deactivated = 4,
}
owned_repr!(NMActiveConnectionState);

/// Active connection state reasons.
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
pub enum NMActiveConnectionStateReason {
    /// The reason for the active connection state change is unknown.
    #[default]
    Unknown = 0,
    /// No reason was given for the active connection state change.
    None = 1,
    /// The active connection changed state because the user disconnected it.
    UserDisconnected = 2,
    /// The active connection changed state because the device it was using was disconnected.
    DeviceDisconnected = 3,
    /// The service providing the VPN connection was stopped.
    ServiceStopped = 4,
    /// The IP config of the active connection was invalid.
    IpConfigInvalid = 5,
    /// The connection attempt to the VPN service timed out.
    ConnectTimeout = 6,
    /// A timeout occurred while starting the service providing the VPN connection.
    ServiceStartTimeout = 7,
    /// Starting the service providing the VPN connection failed.
    ServiceStartFailed = 8,
    /// Necessary secrets for the connection were not provided.
    NoSecrets = 9,
    /// Authentication to the server failed.
    LoginFailed = 10,
    /// The connection was deleted from settings.
    ConnectionRemoved = 11,
    /// Master connection of this connection failed to activate.
    DependencyFailed = 12,
    /// Could not create the software device link.
    DeviceRealizeFailed = 13,
    /// The device this connection depended on disappeared.
    DeviceRemoved = 14,
}
owned_repr!(NMActiveConnectionStateReason);

/// Flags describing the current activation state.
///
/// TODO: Make a bitflags
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
pub enum NMActivationStateFlags {
    /// an alias for numeric zero, no flags set.
    #[default]
    None = 0,
    /// the device is a master.
    IsMaster = 0x1,
    /// the device is a slave.
    IsSlave = 0x2,
    /// layer2 is activated and ready.
    Layer2Ready = 0x4,
    /// IPv4 setting is completed.
    IP4Ready = 0x8,
    /// IPv6 setting is completed.
    IP6Ready = 0x10,
    /// The master has any slave devices attached. This only makes sense if the device is a master.
    MasterHasSlaves = 0x20,
    /// the lifetime of the activation is bound to the visibility of the connection profile,
    /// which in turn depends on "connection.permissions" and whether a session for the user exists. Since: 1.16.
    LifetimeBoundToProfileVisibility = 0x40,
    /// the active connection was generated to represent an external configuration of a networking device. Since: 1.26.
    External = 0x80,
}
owned_repr!(NMActivationStateFlags);

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
    Deserialize,
    Serialize,
    derive_more::From,
    derive_more::Display,
    derive_more::Deref,
    derive_more::AsRef,
)]
pub struct UpSpeed(pub u64);
impl TryFrom<zvariant::OwnedValue> for UpSpeed {
    type Error = zvariant::Error;
    fn try_from(value: zvariant::OwnedValue) -> Result<Self, Self::Error> {
        match value.downcast_ref::<u64>() {
            Ok(v) => Ok(Self(v)),
            _ => Err(zvariant::Error::IncorrectType),
        }
    }
}

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
    Deserialize,
    Serialize,
    derive_more::From,
    derive_more::Display,
    derive_more::Deref,
    derive_more::AsRef,
)]
pub struct DownSpeed(pub u64);
impl TryFrom<zvariant::OwnedValue> for DownSpeed {
    type Error = zvariant::Error;
    fn try_from(value: zvariant::OwnedValue) -> Result<Self, Self::Error> {
        match value.downcast_ref::<u64>() {
            Ok(v) => Ok(Self(v)),
            _ => Err(zvariant::Error::IncorrectType),
        }
    }
}
