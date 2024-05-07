use super::props::*;
use super::proxy_functions::NetworkProxies;
use super::speed::Speed;
use super::xmlgen::{
    access_point::AccessPointProxy,
    active_connection::ActiveProxy,
    device::{DeviceProxy, StatisticsProxy},
    network_manager::NetworkManagerProxy,
    wireless_device::WirelessProxy,
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
    pub nm_proxy: NetworkManagerProxy<'c>,
    pub device_proxy: Option<DeviceProxy<'c>>,
    pub active_proxy: Option<ActiveProxy<'c>>,
    pub ap_proxy: Option<AccessPointProxy<'c>>,

    speed: Option<Speed<'c>>,

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
        device_name: Option<Arc<String>>,
        speed_poll_rate: Option<Duration>,
    ) -> NetResult<Self> {
        if !config.is_enabled() {
            return Err(NetError::NetDisabled);
        }
        if !kill_receiver.is_empty() {
            return Err(NetError::InvalidState(
                "New listener created while kill channel has a value!",
            ));
        }

        let nm_proxy = NetworkManagerProxy::builder(conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let device_proxy_opt;
        let active_proxy_opt;

        match device_name {
            Some(name) => {
                let devices = nm_proxy.devices().await?;
                let device =
                    super::proxy_functions::specified_device_proxy(conn, devices, &name).await?;

                // only use what we need
                let active = if config.active_conn_props() {
                    let path = device.active_connection().await?;
                    Some(super::proxy_functions::active_proxy(conn, path).await?)
                } else {
                    None
                };
                device_proxy_opt = Some(device);
                active_proxy_opt = active;
            }
            None => {
                let path = nm_proxy.primary_connection().await?;
                let active = super::proxy_functions::active_proxy(conn, path).await?;
                let mut device = None;

                if config.device_props() {
                    let active_devices = active.devices().await?;
                    let mut devices = super::proxy_functions::device_proxies(conn, active_devices);

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

                device_proxy_opt = device;
                active_proxy_opt = Some(active);
            }
        }

        let ap_proxy = if config.access_point_props() {
            super::proxy_functions::access_point(
                &conn,
                device_proxy_opt
                    .as_ref()
                    .expect("Device proxy options must be enabled to get access point props!"),
            )
            .await?
        } else {
            None
        };
        todo!();
        // let speed = speed_poll_rate.map(|d| Speed::new(conn, device_path, sender, poll_rate))

        // let me = Self {
        //     nm_proxy,
        //     device_proxy: device_proxy_opt,
        //     active_proxy: active_proxy_opt,
        //     ap_proxy,
        //     kill_receiver,
        //     property_sender,
        // };

        // macro_rules! listener_inner {
        //     ($( $proxy:expr => $( $prop:tt: $prop_type:ident ),+ );+) => {
        //         async {
        //             try_join! {
        //                 $($(
        //                     async {
        //                         property_sender.send(NMPropertyType::$prop_type($proxy.$prop().await?)).await.map_err(|e| {
        //                             error!("Failed to send current '{}' to format receiver: {e}", stringify!($prop));
        //                             NetError::SendError
        //                         })
        //                     }
        //                 ),+),+
        //             }
        //         }
        //     };
        // }

        // let inner = listener_inner![active_proxy => state: ActiveConnectionState; access_point_proxy => ssid: Ssid, strength: Strength];

        // Ok(me)
    }
}
