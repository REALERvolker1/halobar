use super::*;

struct FormatNet {
    pub ssid: String,
    pub interface: String,
    pub up_speed: u64,
    pub down_speed: u64,
    pub signal_strength: u64,

    format: FmtSegmentVec,
}

// config_struct! {
//     [Net]
//     format: FormatStr = FormatStr::default(),
// }

pub struct Network {
    interface: String,
    connection: zbus::Connection,
    channel: BiChannel<String, Event>,
}
