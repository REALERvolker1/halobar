use self::xmlgen::{
    access_point::AccessPointProxy,
    active_connection::ActiveProxy,
    device::{DeviceProxy, StatisticsProxy},
    network_manager::NetworkManagerProxy,
};
use super::*;
use zbus::{proxy::CacheProperties, zvariant::OwnedObjectPath};

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
    pub connection: &'c zbus::Connection,
    pub nm_proxy: NetworkManagerProxy<'c>,
    device_proxy: Option<DeviceProxy<'c>>,
    active_proxy: Option<ActiveProxy<'c>>,
    access_point_proxy: Option<AccessPointProxy<'c>>,

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
    ) -> NetResult<()> {
        if !kill_receiver.is_empty() {
            return Err(NetError::InvalidState(
                "New listener created while kill channel has a value!",
            ));
        }

        let nm_proxy = NetworkManagerProxy::builder(conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        // let device_proxy = if config.iface_name | config.state {}

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

        todo!();
    }
}
