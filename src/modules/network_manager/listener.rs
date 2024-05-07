use super::props::*;
use super::proxy_functions::NetworkProxies;
use super::xmlgen::{
    access_point::AccessPointProxy,
    active_connection::ActiveProxy,
    device::{DeviceProxy, StatisticsProxy},
    network_manager::NetworkManagerProxy,
};
use crate::prelude::*;
use zbus::CacheProperties;

pub(super) struct NetModule<'c> {
    pub device_name: Option<Arc<String>>,
    pub config: Arc<NetKnown>,
    pub nm_proxy: NetworkManagerProxy<'c>,
    // device: Option<DeviceProxy<'c>>,
    // active: Option<ActiveProxy<'c>>,
}
impl<'c> NetModule<'c> {
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

pub(super) struct Listener<'c> {
    nm_proxy: &'c NetworkManagerProxy<'c>,
    proxies: NetworkProxies<'c>,

    pub kill_receiver: Arc<flume::Receiver<()>>,
    property_sender: Arc<mpsc::Sender<NMPropertyType>>,
}
impl<'c> Listener<'c> {
    #[instrument(level = "debug", skip_all)]
    pub async fn new(
        conn: &'c zbus::Connection,
        kill_receiver: Arc<flume::Receiver<()>>,
        property_sender: Arc<mpsc::Sender<NMPropertyType>>,
        config: NMPropertyFlags,
        device_name: Option<&str>,
        nm_proxy: &'c NetworkManagerProxy<'c>,
    ) -> NetResult<Self> {
        if !kill_receiver.is_empty() {
            return Err(NetError::InvalidState(
                "New listener created while kill channel has a value!",
            ));
        }

        let mut me = Self {
            nm_proxy,
            proxies: NetworkProxies::new(conn, nm_proxy, config, device_name).await?,
            kill_receiver,
            property_sender,
        };

        macro_rules! listener_inner {
            ($( $proxy:expr => $( $prop:tt: $prop_type:ident ),+ );+) => {
                async {
                    try_join! {
                        $($(
                            async {
                                property_sender.send(NMPropertyType::$prop_type($proxy.$prop().await?)).await.map_err(|e| {
                                    error!("Failed to send current '{}' to format receiver: {e}", stringify!($prop));
                                    NetError::SendError
                                })
                            }
                        ),+),+
                    }
                }
            };
        }

        // let inner = listener_inner![active_proxy => state: ActiveConnectionState; access_point_proxy => ssid: Ssid, strength: Strength];
        Ok(me)
    }
}
