use super::*;
use xmlgen::{
    active_connection::ActiveProxy,
    device::{DeviceProxy, StatisticsProxy},
    network_manager::NetworkManagerProxy,
};
use zbus::{proxy::CacheProperties, zvariant::OwnedObjectPath};

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
    #[instrument(level = "debug", skip(conn))]
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

#[instrument(level = "debug", skip_all)]
async fn listener<'c>(
    conn: &'c zbus::Connection,
    kill_receiver: Arc<flume::Receiver<()>>,
    property_sender: Arc<mpsc::Sender<NMPropertyType>>,
    config: NMPropertyFlags,
    device_name: Option<&str>,
) -> NetResult<()> {
    if !kill_receiver.is_empty() {
        return Err(NetError::InvalidState(
            "New listener created while kill channel has a value!",
        ));
    }

    // let mut poll_collection = FuturesUnordered::new();

    // if config.speed_props() {
    //     // poll_collection.push(future)
    // }

    todo!();
}

#[instrument(level = "trace", skip_all)]
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

#[instrument(level = "trace", skip_all)]
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
