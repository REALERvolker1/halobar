use self::xmlgen::{
    access_point::AccessPointProxy,
    active_connection::ActiveProxy,
    device::{DeviceProxy, StatisticsProxy},
    network_manager::NetworkManagerProxy,
    wireless_device::WirelessProxy,
};
use super::*;
use data_functions::variants::NMDeviceType;
use zbus::{
    proxy::CacheProperties,
    zvariant::{ObjectPath, OwnedObjectPath},
};

#[derive(Debug)]
pub(super) struct Speed<'c> {
    pub rx_total: u64,
    pub tx_total: u64,

    pub rx_per_second: Size,
    pub tx_per_second: Size,

    pub last_checked: Instant,
    pub proxy: StatisticsProxy<'c>,
}
impl<'c> Speed<'c> {
    #[instrument(level = "debug")]
    pub async fn new(conn: &'c SystemConnection, device_path: OwnedObjectPath) -> NetResult<Self> {
        let proxy = StatisticsProxy::builder(&conn.0)
            .path(device_path)?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let (tx_total, rx_total) = try_join![proxy.tx_bytes(), proxy.rx_bytes()]?;

        Ok(Self {
            rx_total: rx_total.0,
            tx_total: tx_total.0,
            rx_per_second: Size::from_const(0),
            tx_per_second: Size::from_const(0),
            last_checked: Instant::now(),
            proxy,
        })
    }
    #[instrument(level = "trace")]
    pub async fn refresh(&mut self) -> NetResult<()> {
        let (tx_bytes, rx_bytes) = try_join![self.proxy.tx_bytes(), self.proxy.rx_bytes()]?;
        let checked_at = Instant::now();
        let time_interval = self.last_checked.duration_since(checked_at);
        let time_interval_seconds = time_interval.as_secs_f64();

        macro_rules! diff {
            ($type:tt) => {
                ::paste::paste! {{
                    let diff = self.[<$type _total>] - [<$type _bytes>].0;
                    let bytes_per_second = diff as f64 / time_interval_seconds;

                    Size::from_bytes(bytes_per_second)
                }}
            };
        }

        self.rx_per_second = diff!(rx);
        self.tx_per_second = diff!(tx);
        self.tx_total = tx_bytes.0;
        self.rx_total = rx_bytes.0;
        self.last_checked = checked_at;

        Ok(())
    }
}

pub(super) struct NetworkProxies<'c> {
    conn: &'c zbus::Connection,
    nm_proxy: NetworkManagerProxy<'c>,
    device_proxy: Option<DeviceProxy<'c>>,
    active_proxy: Option<ActiveProxy<'c>>,
    ap_proxy: Option<AccessPointProxy<'c>>,
}
impl<'c> NetworkProxies<'c> {
    pub async fn new(
        conn: &'c zbus::Connection,
        listener_config: NMPropertyFlags,
        device_name: Option<&str>,
    ) -> NetResult<Self> {
        let nm_proxy = NetworkManagerProxy::builder(conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let (device_proxy, active_proxy) = if listener_config.device_props()
            | listener_config.active_conn_props()
            | listener_config.access_point_props()
        {
            match device_name {
                Some(name) => {
                    let devices = nm_proxy.devices().await?;
                    let device_proxy = specified_device_proxy(conn, devices, name).await?;

                    // only use what we need
                    let active = if listener_config.active_conn_props() {
                        let path = device_proxy.active_connection().await?;
                        Some(active_proxy(conn, path).await?)
                    } else {
                        None
                    };

                    (Some(device_proxy), active)
                }
                None => {
                    let path = nm_proxy.primary_connection().await?;
                    let active = active_proxy(conn, path).await?;
                    let mut device = None;

                    if listener_config.device_props() {
                        let active_devices = active.devices().await?;
                        let mut devices = device_proxies(conn, active_devices);

                        while let Some(d) = devices.next().await {
                            match d {
                                Ok(proxy) => {
                                    device.replace(proxy);
                                    break;
                                }
                                Err(e) => error!("Error getting device: {e}"),
                            }
                        }
                    }

                    (device, Some(active))
                }
            }
        } else {
            (None, None)
        };

        let ap_proxy = if listener_config.access_point_props() {
            let device_path = device_proxy.expect("Device proxy options must be enabled to get access point props! Please report this as a bug!").inner().path();

            todo!();
        };

        todo!();
    }
}

// #[instrument(level = "trace", skip(conn))]
// pub async fn wifi_access_point<'c>(conn: &'c zbus::Connection, path: &'c ObjectPath) -> zbus::Result<Option<AccessPointProxy<'c>>> {
//     let proxy = WirelessProxy::builder(conn)
// }

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
fn device_proxies<'c>(
    conn: &'c zbus::Connection,
    devices: Vec<OwnedObjectPath>,
) -> FuturesUnordered<impl std::future::Future<Output = zbus::Result<DeviceProxy<'c>>>> {
    devices
        .into_iter()
        .map(|p| device_proxy(conn, p))
        .collect::<FuturesUnordered<_>>()
}

#[instrument(level = "trace", skip(conn))]
async fn specified_device_proxy<'c>(
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
