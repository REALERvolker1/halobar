use super::variants::NMActiveConnectionState;
use crate::prelude::{Deserialize, Serialize};

#[derive(Debug, strum_macros::Display, Serialize, Deserialize)]
pub enum NMPropertyType {
    UpSpeed(u64),
    DownSpeed(u64),
    Ssid(String),
    IfaceName(String),
    Strength(u8),
    ActiveConnectionState(NMActiveConnectionState),
}
