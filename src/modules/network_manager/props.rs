use super::variants::{NMActiveConnectionState, NMState};
use crate::prelude::{config_struct, data_flags, Deserialize, Serialize};

#[derive(Debug, strum_macros::Display, Serialize, Deserialize, strum_macros::EnumDiscriminants)]
pub enum NMPropertyType {
    UpSpeed(u64),
    DownSpeed(u64),
    Ssid(String),
    IfaceName(String),
    Strength(u8),
    ActiveConnectionState(NMActiveConnectionState),
}

data_flags! {
    pub(super) NMPropertyFlags => super::props::NMPropertyTypeDiscriminants {
        up_speed => UpSpeed,
        down_speed => DownSpeed,
        ssid => Ssid,
        iface_name => IfaceName,
        strength => Strength,
        state => ActiveConnectionState,
    }
}

config_struct! {
    [Net]
    id: u16 = 0,
    // format: FormatStr = FormatStr::default(),
    poll_rate_seconds: u64 = 5,
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
