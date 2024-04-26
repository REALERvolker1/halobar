//! # D-Bus interface proxy for: `org.freedesktop.NetworkManager.Device.Statistics`
//!
//! This code was generated by `zbus-xmlgen` `4.1.0` from D-Bus introspection data.
//! Source: `Interface '/org/freedesktop/NetworkManager/Devices/1' from service 'org.freedesktop.NetworkManager' on system bus`.
//!
//! You may prefer to adapt it, instead of using it verbatim.
//!
//! More information can be found in the [Writing a client proxy] section of the zbus
//! documentation.
//!
//! This type implements the [D-Bus standard interfaces], (`org.freedesktop.DBus.*`) for which the
//! following zbus API can be used:
//!
//! * [`zbus::fdo::PropertiesProxy`]
//! * [`zbus::fdo::IntrospectableProxy`]
//! * [`zbus::fdo::PeerProxy`]
//!
//! Consequently `zbus-xmlgen` did not generate code for the above interfaces.
//!
//! [Writing a client proxy]: https://dbus2.github.io/zbus/client.html
//! [D-Bus standard interfaces]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces,
use zbus::proxy;
#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Statistics",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Devices/1"
)]
trait Statistics {
    /// RefreshRateMs property
    #[zbus(property)]
    fn refresh_rate_ms(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn set_refresh_rate_ms(&self, value: u32) -> zbus::Result<()>;

    /// RxBytes property
    #[zbus(property)]
    fn rx_bytes(&self) -> zbus::Result<u64>;

    /// TxBytes property
    #[zbus(property)]
    fn tx_bytes(&self) -> zbus::Result<u64>;
}
//! # D-Bus interface proxy for: `org.freedesktop.NetworkManager.Device`
//!
//! This code was generated by `zbus-xmlgen` `4.1.0` from D-Bus introspection data.
//! Source: `Interface '/org/freedesktop/NetworkManager/Devices/1' from service 'org.freedesktop.NetworkManager' on system bus`.
//!
//! You may prefer to adapt it, instead of using it verbatim.
//!
//! More information can be found in the [Writing a client proxy] section of the zbus
//! documentation.
//!
//! This type implements the [D-Bus standard interfaces], (`org.freedesktop.DBus.*`) for which the
//! following zbus API can be used:
//!
//! * [`zbus::fdo::PropertiesProxy`]
//! * [`zbus::fdo::IntrospectableProxy`]
//! * [`zbus::fdo::PeerProxy`]
//!
//! Consequently `zbus-xmlgen` did not generate code for the above interfaces.
//!
//! [Writing a client proxy]: https://dbus2.github.io/zbus/client.html
//! [D-Bus standard interfaces]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces,
use zbus::proxy;
#[proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Devices/1"
)]
trait Device {
    /// Delete method
    fn delete(&self) -> zbus::Result<()>;

    /// Disconnect method
    fn disconnect(&self) -> zbus::Result<()>;

    /// GetAppliedConnection method
    fn get_applied_connection(
        &self,
        flags: u32,
    ) -> zbus::Result<(
        std::collections::HashMap<
            String,
            std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
        >,
        u64,
    )>;

    /// Reapply method
    fn reapply(
        &self,
        connection: std::collections::HashMap<
            &str,
            std::collections::HashMap<&str, &zbus::zvariant::Value<'_>>,
        >,
        version_id: u64,
        flags: u32,
    ) -> zbus::Result<()>;

    /// StateChanged signal
    #[zbus(signal)]
    fn state_changed(&self, new_state: u32, old_state: u32, reason: u32) -> zbus::Result<()>;

    /// ActiveConnection property
    #[zbus(property)]
    fn active_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Autoconnect property
    #[zbus(property)]
    fn autoconnect(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_autoconnect(&self, value: bool) -> zbus::Result<()>;

    /// AvailableConnections property
    #[zbus(property)]
    fn available_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    /// Capabilities property
    #[zbus(property)]
    fn capabilities(&self) -> zbus::Result<u32>;

    /// DeviceType property
    #[zbus(property)]
    fn device_type(&self) -> zbus::Result<u32>;

    /// Dhcp4Config property
    #[zbus(property)]
    fn dhcp4_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Dhcp6Config property
    #[zbus(property)]
    fn dhcp6_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Driver property
    #[zbus(property)]
    fn driver(&self) -> zbus::Result<String>;

    /// DriverVersion property
    #[zbus(property)]
    fn driver_version(&self) -> zbus::Result<String>;

    /// FirmwareMissing property
    #[zbus(property)]
    fn firmware_missing(&self) -> zbus::Result<bool>;

    /// FirmwareVersion property
    #[zbus(property)]
    fn firmware_version(&self) -> zbus::Result<String>;

    /// HwAddress property
    #[zbus(property)]
    fn hw_address(&self) -> zbus::Result<String>;

    /// Interface property
    #[zbus(property)]
    fn interface(&self) -> zbus::Result<String>;

    /// InterfaceFlags property
    #[zbus(property)]
    fn interface_flags(&self) -> zbus::Result<u32>;

    /// Ip4Address property
    #[zbus(property)]
    fn ip4_address(&self) -> zbus::Result<u32>;

    /// Ip4Config property
    #[zbus(property)]
    fn ip4_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Ip4Connectivity property
    #[zbus(property)]
    fn ip4_connectivity(&self) -> zbus::Result<u32>;

    /// Ip6Config property
    #[zbus(property)]
    fn ip6_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Ip6Connectivity property
    #[zbus(property)]
    fn ip6_connectivity(&self) -> zbus::Result<u32>;

    /// IpInterface property
    #[zbus(property)]
    fn ip_interface(&self) -> zbus::Result<String>;

    /// LldpNeighbors property
    #[zbus(property)]
    fn lldp_neighbors(
        &self,
    ) -> zbus::Result<Vec<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>>;

    /// Managed property
    #[zbus(property)]
    fn managed(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_managed(&self, value: bool) -> zbus::Result<()>;

    /// Metered property
    #[zbus(property)]
    fn metered(&self) -> zbus::Result<u32>;

    /// Mtu property
    #[zbus(property)]
    fn mtu(&self) -> zbus::Result<u32>;

    /// NmPluginMissing property
    #[zbus(property)]
    fn nm_plugin_missing(&self) -> zbus::Result<bool>;

    /// Path property
    #[zbus(property)]
    fn path(&self) -> zbus::Result<String>;

    /// PhysicalPortId property
    #[zbus(property)]
    fn physical_port_id(&self) -> zbus::Result<String>;

    /// Ports property
    #[zbus(property)]
    fn ports(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    /// Real property
    #[zbus(property)]
    fn real(&self) -> zbus::Result<bool>;

    /// State property
    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;

    /// StateReason property
    #[zbus(property)]
    fn state_reason(&self) -> zbus::Result<(u32, u32)>;

    /// Udi property
    #[zbus(property)]
    fn udi(&self) -> zbus::Result<String>;
}
