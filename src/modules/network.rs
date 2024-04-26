mod access_point;
mod active_connection;
mod device;
mod network_manager;
mod settings;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetData {
    // TODO: Figure out how to get this
    // pub ssid: Arc<String>,
    pub up_speed: u64,
    pub down_speed: u64,
    pub is_online: bool,
}

struct FormatNet {
    data: NetData,
    /// TODO: Allow choosing decimal rounding
    format: FmtSegmentVec,
    format_offline: FmtSegmentVec,
}
// impl HaloFormatter for FormatNet {
//     type Data = NetData;
//     fn current_data<'a>(&'a self) -> &'a Self::Data {
//         &self.data
//     }
//     fn default_format_str() -> FormatStr {
//         ""
//     }
// }

// config_struct! {
//     [Net]
//     format: FormatStr = FormatStr::default(),
// }

pub struct Network {
    /// The device in /sys/class/net
    interface: Arc<String>,

    last_data: NetData,
    connection: zbus::Connection,
    channel: BiChannel<String, Event>,
}
impl Network {
    // fn refresh(&mut self) -> Result<(), NetError> {
    //     // I get that doing it in this order is a bit more innacurate, but I would rather overestimate than underestimate in this instance.
    //     let last_checked = Instant::now();
    //     let since_last = last_checked.duration_since(self.last_checked);
    //     self.last_checked = last_checked;
    //     let seconds = since_last.as_secs();

    //     let data = NetData {
    //         up_speed: Self::speed_difference(seconds, self.last_data.up_speed, &self.tx_packets)?,
    //         down_speed: Self::speed_difference(
    //             seconds,
    //             self.last_data.down_speed,
    //             &self.rx_packets,
    //         )?,
    //     };

    //     Ok(())
    // }
    // /// Quick and dirty way to query one of the tx or rx things
    // fn speed_difference(time_seconds: u64, previous: u64, path: &Path) -> Result<u64, NetError> {
    //     let current = fs::read_to_string(path)?.parse::<u64>()?;
    //     let difference = current.saturating_sub(previous);

    //     let size_bytes = difference / time_seconds;
    //     Ok(size_bytes)
    // }
}
// impl BackendModule for Network

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("Invalid interface: {0}")]
    InvalidInterface(PathBuf),
    #[error("Nix errno: {0}")]
    Errno(#[from] Errno),
    #[error("{0}")]
    Io(#[from] tokio::io::Error),
    #[error("Error parsing integer: {0}")]
    Parse(#[from] std::num::ParseIntError),
}
