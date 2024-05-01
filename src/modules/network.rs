mod xmlgen;

// mod chosen;
mod variants;

use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;
use zbus::proxy::{CacheProperties, PropertyStream};

use self::{
    variants::{NMDeviceType, NMState},
    xmlgen::network_manager,
};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetData {
    pub ssid: Option<Arc<String>>,
    pub device: Option<Arc<String>>,
    pub up_speed: Option<u64>,
    pub down_speed: Option<u64>,
    pub state: NMState,
}

config_struct! {
    [NetIcon]
    asleep: char = '󰲚',
    connected_global: char = '󰱔',
    connected_local: char = '󰲁',
    connected_site: char = '󰲝',
    connecting: char = '󰲺',
    disconnected: char = '󰲜',
    disconnecting: char = '󰲝',
    unknown: char = '󰲊',
}

impl NetIconKnown {
    /// TODO: Icon config
    fn state_icon(&self, state: NMState) -> char {
        match state {
            NMState::Asleep => self.asleep,
            NMState::ConnectedGlobal => self.connected_global,
            NMState::ConnectedLocal => self.connected_local,
            NMState::ConnectedSite => self.connected_site,
            NMState::Connecting => self.connecting,
            NMState::Disconnected => self.disconnected,
            NMState::Disconnecting => self.disconnecting,
            NMState::Unknown => self.unknown,
        }
    }
    fn is_online(state: NMState) -> bool {
        match state {
            NMState::ConnectedGlobal | NMState::Unknown => true,
            _ => false,
        }
    }
}

// struct FormatNet {
//     data: NetData,
//     /// TODO: Allow choosing decimal rounding
//     format: FmtSegmentVec,
//     format_offline: FmtSegmentVec,
// }
// impl HaloFormatter for FormatNet {
//     type Data = NetData;
//     fn current_data<'a>(&'a self) -> &'a Self::Data {
//         &self.data
//     }
//     fn default_format_str() -> FormatStr {
//         "{icon} {up_speed} UP, {down_speed} DOWN".into()
//     }
//     fn fn_table<'a>(&'a self) -> halobar_config::fmt::FnTable<Self::Data, 1> {
//         FnTable([
//             ("icon", |data| Some(data.state.state_icon().to_string())),
//             ("up_speed", |data| Some(format!("{}", data.up_speed))),
//             ("down_speed", |data| Some(format!("{}", data.down_speed))),
//         ])
//     }
//     fn segments<'s>(&'s self) -> FmtSegments<'s> {
//         if self.data.
//     }
// }

config_struct! {
    [Net]
    show_speed_up: bool = true,
    show_speed_down: bool = true,
    show_ssid: bool = true,
    show_device: bool = true,
    show_state: bool = true,
    // format: FormatStr = FormatStr::default(),
}
impl NetKnown {
    pub fn is_valid(&self) -> bool {
        self.show_device
            && self.show_speed_down
            && self.show_speed_up
            && self.show_ssid
            && self.show_state
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("Invalid interface: {0}")]
    InvalidInterface(String),
    #[error("Nix errno: {0}")]
    Errno(#[from] Errno),
    #[error("{0}")]
    Io(#[from] tokio::io::Error),
    #[error("Error parsing integer: {0}")]
    Parse(#[from] std::num::ParseIntError),
    #[error("zbus error: {0}")]
    Zbus(#[from] zbus::Error),
    #[error("Networking disabled")]
    NetDisabled,
    #[error("Failed to send message to channel")]
    SendError,
    #[error("Failed to receive message from channel")]
    RecvError,
}

pub struct Network {
    // /// The device in /sys/class/net
    // interface: Option<Arc<String>>,

    // connection: zbus::Connection,
    // channel: BiChannel<NetData, Event>,

    // show_speed_up: bool,
    // show_speed_down: bool,
    // show_ssid: bool,
    // show_device: bool,
    // show_state: bool,
}
impl Network {
    pub async fn init(
        runtime: tokio::runtime::Runtime,
        config: NetKnown,
        conn: zbus::Connection,
        sender: oneshot::Sender<BiChannel<Event, NetData>>,
    ) -> Result<(), NetError> {
        if !config.is_valid() {
            return Err(NetError::NetDisabled);
        }

        let network_manager = network_manager::NetworkManagerProxy::builder(&conn)
            .build()
            .await?;

        let primary = network_manager.primary_connection().await?;

        let mut primary_stream = network_manager.receive_primary_connection_changed().await;

        let (kill_sender, r) = flume::bounded(1);
        // let kill_sender = Arc::new(s);
        let kill_receiver = Arc::new(r);

        let (s, mut property_receiver) = mpsc::channel(8);
        let property_sender = Arc::new(s);

        let format_thread =
            runtime.spawn(async move { while let Some(prop) = property_receiver.recv().await {} });

        while let Some(maybe_path) = primary_stream.next().await {
            // Send the kill error before getting the current connection path, because then I don't have
            // to worry about weird mut references and whatnot.
            kill_sender.send_async(()).await.map_err(|e| {
                error!("Failed to send kill signal to networkmanager modules: {e}");
                NetError::SendError
            })?;

            let path = maybe_path.get().await?;
            let active_proxy = xmlgen::active_connection::ActiveProxy::builder(&conn)
                .path(path)?
                .build()
                .await?;

            let prop_stream = active_proxy.receive_state_changed().await;
            let kill = kill_receiver.clone();
            let prop_sender = property_sender.clone();
            runtime.spawn(async move {
                if let Err(e) =
                    sub_listener("Networkmanager State", prop_stream, kill, prop_sender).await
                {
                    error!("State handler NAME returned error: {e}");
                }
            });

            // primary_stream
        }

        Ok(())
    }
}

async fn sub_listener<'p, T>(
    name: &'static str,
    mut stream: PropertyStream<'p, T>,
    shutdown_receiver: Arc<flume::Receiver<()>>,
    property_sender: Arc<mpsc::Sender<PropertyType>>,
) -> Result<(), NetError>
where
    T: std::marker::Unpin + TryFrom<zvariant::OwnedValue> + std::fmt::Debug + Into<PropertyType>,
    T::Error: Into<zbus::Error>,
{
    loop {
        select! { biased;
            res = shutdown_receiver.recv_async() => {
                if let Err(e) = res {
                    error!("Shutdown receiver for '{name}' returned an error: {e}");
                    return Err(NetError::RecvError);
                }
            }
            Some(s) = stream.next() => {
                let raw_value = s.get().await?;

                let prop = raw_value.into();

                if let Err(e) = property_sender.send(prop).await {
                    error!("Failed to send prop to '{name}' receiver: {e}");
                    return Err(NetError::SendError)
                }
            }
        }
    }

    // Ok(())
}

#[derive(Debug, strum_macros::Display, derive_more::From)]
enum PropertyType {
    UpSpeed(variants::UpSpeed),
    DownSpeed(variants::DownSpeed),
    Ssid(variants::Ssid),
    IfaceName(String),
    ActiveConnectionState(variants::NMActiveConnectionState),
}
