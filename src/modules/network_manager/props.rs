use super::variants::NMActiveConnectionState;
use crate::prelude::{data_flags, Deserialize, Serialize};

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
