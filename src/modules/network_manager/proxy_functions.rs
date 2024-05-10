use super::props::*;
use super::variants::*;
use super::xmlgen::{
    access_point::AccessPointProxy,
    active_connection::ActiveProxy,
    device::{DeviceProxy, StatisticsProxy},
    network_manager::NetworkManagerProxy,
    wireless_device::WirelessProxy,
};
use crate::prelude::*;

use zbus::zvariant::ObjectPath;
use zbus::{proxy::CacheProperties, zvariant::OwnedObjectPath};

/// Get the dbus object path of the proxy. Basically a cloning, use this carefully.
#[inline]
pub(super) fn proxy_path(proxy: &zbus::Proxy<'_>) -> OwnedObjectPath {
    proxy.path().to_owned().into()
}

/// Naive autodetection
pub(super) async fn autodetect_device_name<'c>(
    conn: &'c zbus::Connection,
    nm_proxy: &NetworkManagerProxy<'c>,
) -> NetResult<String> {
    let mut connections = nm_proxy
        .devices()
        .await?
        .into_iter()
        .map(|d| async {
            let proxy = super::proxy_functions::device_proxy(conn, d).await?;

            let dev_type = proxy.device_type().await?;

            // TODO: Possibly add more types
            match dev_type {
                NMDeviceType::Loopback | NMDeviceType::Unknown => {
                    return Err(NetError::NetDisabled)
                }
                _ => {}
            }

            // afaik it will err if it does not have an active conn
            let active_conn = proxy.active_connection().await?;

            let name = proxy.interface().await?;

            debug!("Found active connection at '{active_conn}' for device {name}");

            Ok::<_, NetError>(name)
        })
        .collect::<FuturesUnordered<_>>();

    while let Some(maybe_active) = connections.next().await {
        match maybe_active {
            // just get the first, this is a naive impl, remember!
            Ok(a) => return Ok(a),
            Err(e) => {
                warn!("{e}");
                continue;
            }
        }
    }

    Err(NetError::NetDisabled)
}

pub(super) async fn access_point<'c>(
    conn: &'c zbus::Connection,
    device_path: OwnedObjectPath,
) -> NetResult<Option<AccessPointProxy<'c>>> {
    debug!("Getting access points for device at path '{device_path}'");

    let wireless_proxy = WirelessProxy::builder(conn)
        .path(device_path)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    let access_point = wireless_proxy.active_access_point().await?;
    debug!("Active access point at path '{access_point}'");

    todo!();
}

/// A constructor for an `ActiveProxy`, to have one source of truth
#[instrument(level = "trace", skip(conn))]
pub async fn active_proxy<'c>(
    conn: &'c zbus::Connection,
    path: OwnedObjectPath,
) -> zbus::Result<ActiveProxy<'c>> {
    let proxy = ActiveProxy::builder(conn)
        .path(path)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    Ok(proxy)
}

/// A constructor for a `DeviceProxy`, to have one source of truth
#[instrument(level = "trace", skip(conn))]
pub async fn device_proxy<'c>(
    conn: &'c zbus::Connection,
    path: OwnedObjectPath,
) -> zbus::Result<DeviceProxy<'c>> {
    let proxy = DeviceProxy::builder(conn)
        .path(path)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    Ok(proxy)
}

#[inline]
#[instrument(level = "trace", skip_all)]
pub(super) fn device_proxies<'c>(
    conn: &'c zbus::Connection,
    devices: Vec<OwnedObjectPath>,
) -> FuturesUnordered<impl std::future::Future<Output = zbus::Result<DeviceProxy<'c>>>> {
    devices
        .into_iter()
        .map(|p| device_proxy(conn, p))
        .collect::<FuturesUnordered<_>>()
}

#[instrument(level = "trace", skip(conn))]
pub(super) async fn specified_device_proxy<'c>(
    conn: &'c zbus::Connection,
    devices: Vec<OwnedObjectPath>,
    device_name: &str,
) -> NetResult<DeviceProxy<'c>> {
    let mut output_proxy = None;

    let mut proxies = device_proxies(conn, devices);

    while let Some(maybe_proxy) = proxies.next().await {
        match maybe_proxy {
            Ok(proxy) => {
                let interface = proxy.interface().await?;
                if interface == device_name {
                    output_proxy.replace(proxy);
                    break;
                } else {
                    info!("Found non-specified device '{interface}', skipping");
                }
            }
            Err(e) => {
                error!("Error creating device proxy: {e}");
                return Err(NetError::Zbus(e));
            }
        }
    }

    let proxy = match output_proxy {
        Some(p) => p,
        None => {
            error!("Could not find device with name '{device_name}'");
            return Err(NetError::InvalidInterface(device_name.to_owned()));
        }
    };

    Ok(proxy)
}
