use super::props::*;
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
    pub config: Arc<NetKnown>,
    pub nm_proxy: NetworkManagerProxy<'c>,
    pub kill_sender: flume::Sender<()>,
    pub kill_receiver: Arc<flume::Receiver<()>>,
}
impl<'c> NetModule<'c> {
    /// Create the active
    #[instrument(level = "debug", skip(conn))]
    pub async fn new(conn: &'c zbus::Connection, mut config: NetKnown) -> NetResult<Self> {
        // if it was killed already, just skip it!
        // TODO: Move into individual device/conn listener

        let config_flags = NMPropertyFlags::from_segments(config.format.segments());

        if !config_flags.is_enabled() {
            return Err(NetError::NetDisabled);
        }

        let nm_proxy = NetworkManagerProxy::builder(conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        if config.device.is_empty() {
            config.device = super::proxy_functions::autodetect_device_name(conn, &nm_proxy).await?;
        }

        let (kill_sender, rec) = flume::bounded(1);

        Ok(Self {
            config: Arc::new(config),
            nm_proxy,
            kill_sender,
            kill_receiver: Arc::new(rec),
        })
    }
    pub async fn run(&mut self) -> NetResult<()> {
        let mut state_stream = self.nm_proxy.receive_state_changed().await;

        while let Some(state) = state_stream.next().await {
            let new = state.get().await?;
            info!("{}", new);
        }

        Err(NetError::EarlyReturn)
    }
}

pub(super) struct Listener<'c> {
    pub device_path: OwnedObjectPath,

    pub nm_proxy: NetworkManagerProxy<'c>,
    pub device_proxy: DeviceProxy<'c>,
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
        device_name: &str,
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

        let devices = nm_proxy.devices().await?;
        let device_proxy =
            super::proxy_functions::specified_device_proxy(conn, devices, device_name).await?;

        let device_path = super::proxy_functions::proxy_path(device_proxy.inner());

        // only use what we need
        let active_proxy = if config.active_conn_props() {
            let path = device_proxy.active_connection().await?;
            Some(super::proxy_functions::active_proxy(conn, path).await?)
        } else {
            None
        };

        let ap_proxy = if config.access_point_props() {
            super::proxy_functions::access_point(&conn, device_path.clone()).await?
        } else {
            None
        };

        let speed = match speed_poll_rate {
            Some(d) => {
                Some(Speed::new(conn, device_path.clone(), Arc::clone(&property_sender), d).await?)
            }
            None => None,
        };

        let me = Self {
            device_path,
            nm_proxy,
            device_proxy,
            active_proxy,
            ap_proxy,
            kill_receiver,
            property_sender,
            speed,
        };

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

        Ok(me)
    }
}
