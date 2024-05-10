use super::variants::{NM80211Mode, NMActiveConnectionState, NMState};
use crate::prelude::{
    config_struct, data_flags, mpsc, Arc, Deserialize, Deserialize_repr, FmtSegmentVec, Serialize,
    Serialize_repr,
};

#[derive(Debug, strum_macros::Display, Serialize, Deserialize, strum_macros::EnumDiscriminants)]
pub enum NMPropertyType {
    UpSpeed(u64),
    DownSpeed(u64),
    Ssid(String),
    IfaceName(String),
    Strength(u8),
    ActiveConnectionState(NMActiveConnectionState),
    Mode(NM80211Mode),
}

data_flags! {
    pub(super) NMPropertyFlags => super::props::NMPropertyTypeDiscriminants {
        up_speed => UpSpeed,
        down_speed => DownSpeed,
        ssid => Ssid,
        iface_name => IfaceName,
        strength => Strength,
        state => ActiveConnectionState,
        mode => Mode,
    }
}
impl NMPropertyFlags {
    /// Whether to create the network speed polling listener
    #[inline]
    pub fn speed_props(&self) -> bool {
        self.up_speed | self.down_speed
    }
    /// Whether to show the active wifi access point
    #[inline]
    pub fn access_point_props(&self) -> bool {
        self.ssid | self.strength | self.mode
    }
    /// Whether to create/use the active device proxy or not
    #[inline]
    pub fn device_props(&self) -> bool {
        self.speed_props() | self.iface_name | self.access_point_props()
    }
    /// Whether to create/use the active connection proxy or not
    #[inline]
    pub fn active_conn_props(&self) -> bool {
        self.state
    }
}

config_struct! {
    [Net]
    id: u16 = 0,
    device: String = String::new(),
    poll_rate_seconds: u64 = 5,
    // TODO: Make a real good default
    format: FmtSegmentVec = FmtSegmentVec::new("{state?$ }{up_speed}/{down_speed} {strength?Strength: $:No strength}").unwrap(),
}

config_struct! {
    [NMIcons]
    asleep: char = '󰲚',
    connected_global: char = '󰱔',
    connected_local: char = '󰲁',
    connected_site: char = '󰲝',
    connecting: char = '󰲺',
    disconnected: char = '󰲜',
    disconnecting: char = '󰲝',
    unknown: char = '󰲊',
}
impl NMIconsKnown {
    /// TODO: Icon config
    pub fn state_icon(&self, state: NMState) -> char {
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
    pub fn is_online(state: NMState) -> bool {
        match state {
            NMState::ConnectedGlobal | NMState::Unknown => true,
            _ => false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("Invalid interface: {0}")]
    InvalidInterface(String),
    #[error("{0}")]
    Io(#[from] tokio::io::Error),
    #[error("Error parsing integer: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("zbus error: {0}")]
    Zbus(#[from] zbus::Error),
    #[error("Networking disabled")]
    NetDisabled,
    #[error("Failed to send message to channel")]
    SendError,
    #[error("Failed to receive message from channel")]
    RecvError,
    #[error("Failed to join task")]
    JoinError,
    #[error("Invalid state detected: {0}")]
    InvalidState(&'static str),
    #[error("Returned early!")]
    EarlyReturn,
}

macro_rules! from_send {
    ($enum:ident :: $enum_kind:tt: $( $err_type:ty $( = $generics:tt )? ),+) => {
        $(
            impl$(<$generics>)? From<$err_type> for $enum {
                #[inline]
                fn from(_: $err_type) -> Self {
                    Self::$enum_kind
                }
            }
        )+
    };
}

from_send! [NetError :: SendError: flume::SendError<T> = T, flume::TrySendError<T> = T, flume::SendTimeoutError<T> = T, mpsc::error::SendError<T> = T];
from_send! [NetError :: SendError: flume::RecvError, flume::TryRecvError, flume::RecvTimeoutError, mpsc::error::TryRecvError];

pub type NetResult<T> = std::result::Result<T, NetError>;
