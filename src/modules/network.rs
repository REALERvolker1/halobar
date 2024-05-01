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
    #[error("Failed to send network module channel to subscriber")]
    InitializerSendError,
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

        let (s, r) = flume::bounded(1);
        let sender = Arc::new(s);
        let receiver = Arc::new(r);

        while let Some(conn) = primary_stream.next().await {
            let conn_path = conn.get().await?;

            primary_stream
        }

        Ok(())
    }
}

struct SubListener<'p, T> {
    stream: PropertyStream<'p, T>,
    prop_type: PropertyTypeDiscriminants,
    shutdown_receiver: Arc<flume::Receiver<()>>,
    property_sender: Arc<mpsc::Sender<PropertyType>>,
}

#[derive(Debug, strum_macros::EnumDiscriminants)]
enum PropertyType {
    UpSpeed(u64),
    DownSpeed(u64),
}
