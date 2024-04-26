use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetData {
    pub ssid: Arc<String>,
    pub up_speed: u64,
    pub down_speed: u64,
}

struct FormatNet {
    data: NetData,

    format: FmtSegmentVec,
}

// config_struct! {
//     [Net]
//     format: FormatStr = FormatStr::default(),
// }

pub struct Network {
    interface: Arc<String>,
    // connection: zbus::Connection,
    channel: BiChannel<String, Event>,
    networks: sysinfo::Networks,
    is_connected: bool,
    last_checked: Instant,
}
impl Network {
    fn refresh(&mut self) -> Result<(), NetError> {
        self.networks.refresh_list();
        self.networks.refresh();
        let network = match self.networks.get(self.interface.as_ref()) {
            Some(i) => i,
            None => return Err(NetError::InvalidInterface(self.interface.clone())),
        };
        let last_checked = Instant::now();
        let since_last = last_checked.duration_since(self.last_checked);
        self.last_checked = last_checked;

        // let data = NetData {
        //     ssid
        // };
        // network.received()
        

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("Invalid interface: {0}")]
    InvalidInterface(Arc<String>),
    #[error("Nix errno: {0}")]
    Errno(#[from] Errno),
}
