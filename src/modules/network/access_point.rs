//! # D-Bus interface proxy for: `org.freedesktop.NetworkManager.AccessPoint`
//!
//! This code was generated by `zbus-xmlgen` `4.1.0` from D-Bus introspection data.
//! Source: `Interface '/org/freedesktop/NetworkManager/AccessPoint/1' from service 'org.freedesktop.NetworkManager' on system bus`.
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
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/AccessPoint/1"
)]
trait AccessPoint {
    /// Bandwidth property
    #[zbus(property)]
    fn bandwidth(&self) -> zbus::Result<u32>;

    /// Flags property
    #[zbus(property)]
    fn flags(&self) -> zbus::Result<u32>;

    /// Frequency property
    #[zbus(property)]
    fn frequency(&self) -> zbus::Result<u32>;

    /// HwAddress property
    #[zbus(property)]
    fn hw_address(&self) -> zbus::Result<String>;

    /// LastSeen property
    #[zbus(property)]
    fn last_seen(&self) -> zbus::Result<i32>;

    /// MaxBitrate property
    #[zbus(property)]
    fn max_bitrate(&self) -> zbus::Result<u32>;

    /// Mode property
    #[zbus(property)]
    fn mode(&self) -> zbus::Result<u32>;

    /// RsnFlags property
    #[zbus(property)]
    fn rsn_flags(&self) -> zbus::Result<u32>;

    /// Ssid property
    #[zbus(property)]
    fn ssid(&self) -> zbus::Result<Vec<u8>>;

    /// Strength property
    #[zbus(property)]
    fn strength(&self) -> zbus::Result<u8>;

    /// WpaFlags property
    #[zbus(property)]
    fn wpa_flags(&self) -> zbus::Result<u32>;
}
