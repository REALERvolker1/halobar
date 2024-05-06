use self::xmlgen::active_connection::ActiveProxy;

use super::*;
use listener::xmlgen::network_manager::NetworkManagerProxy;
use xmlgen::device::{DeviceProxy, StatisticsProxy};
use zbus::{proxy::CacheProperties, zvariant::OwnedObjectPath};

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

pub(super) struct Listener<'c> {
    pub device_name: Option<Arc<String>>,
    pub config: Arc<NetKnown>,
    pub kill_receiver: Arc<flume::Receiver<()>>,
    property_sender: Arc<mpsc::Sender<NMPropertyType>>,
    pub nm_proxy: NetworkManagerProxy<'c>,
    // device: Option<DeviceProxy<'c>>,
    // active: Option<ActiveProxy<'c>>,
}
impl<'c> Listener<'c> {
    /// Create the active
    pub async fn new(conn: &'c zbus::Connection, config: NetKnown) -> NetResult<Option<Self>> {
        // if it was killed already, just skip it!
        // TODO: Move into individual device/conn listener

        let config_flags = NMPropertyFlags::from_segments(config.format.segments());

        if !config_flags.is_enabled() {
            return Ok(None);
        }

        let nm_proxy = NetworkManagerProxy::builder(conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let device_name = if config.device.is_empty() {
            None
        } else {
            Some(Arc::clone(&config.device))
        };

        todo!();
    }
}

async fn listener<'c>(
    conn: &'c zbus::Connection,
    kill_receiver: Arc<flume::Receiver<()>>,
    property_sender: Arc<mpsc::Sender<NMPropertyType>>,
) -> NetResult<()> {
    if !kill_receiver.is_empty() {
        return Err(NetError::InvalidState(
            "New listener created while kill channel has a value!",
        ));
    }
    todo!();
}

async fn active_from_device<'c>(
    conn: &'c zbus::Connection,
    proxy: DeviceProxy<'c>,
) -> NetResult<ActiveProxy<'c>> {
    let path = proxy.active_connection().await?;
    trace!("Got active connection path: {path}");

    let active = ActiveProxy::builder(conn)
        .path(path)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    Ok(active)
}

async fn device_from_active<'c>(
    conn: &'c zbus::Connection,
    proxy: ActiveProxy<'c>,
    device_name: &str,
) -> NetResult<DeviceProxy<'c>> {
    let devices = proxy.devices().await?;
    let mut output_proxy = None;

    let mut proxies = devices
        .into_iter()
        .map(|p| DeviceProxy::builder(conn).path(p))
        .filter_map(|r| match r {
            Ok(b) => Some(b.cache_properties(CacheProperties::No).build()),
            Err(e) => {
                error!("Could not create device proxy, invalid path: {e}");
                None
            }
        })
        .collect::<FuturesUnordered<_>>();

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

    trace!("Got active connection for device {device_name}");

    Ok(proxy)
}
