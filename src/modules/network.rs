mod xmlgen;

// mod chosen;
mod variants;

use zbus::{
    proxy::{CacheProperties, PropertyStream},
    zvariant::OwnedObjectPath,
};

use self::{
    variants::{NMDeviceType, NMState},
    xmlgen::network_manager,
};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NetData {
    pub ssid: Option<Arc<String>>,
    pub device: Option<Arc<String>>,
    pub up_speed: Option<u64>,
    pub down_speed: Option<u64>,
    pub state: NMState,

    pub is_online: bool,
}
impl NetData {
    pub fn format_task(
        &mut self,
        format_str: &str,
        output_sender: mpsc::Sender<String>,
        mut property_receiver: mpsc::Receiver<PropertyType>,
    ) -> Result<(), NetError> {
        // let formatter = dyn_fmt::Arguments::new(format_str, )
        todo!();
    }
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
// impl HaloFormatter<5> for FormatNet {
//     type Data = NetData;
//     fn current_data<'a>(&'a self) -> &'a Self::Data {
//         &self.data
//     }
//     fn default_format_str() -> FormatStr {
//         "{icon} {up_speed} UP, {down_speed} DOWN".to_owned().into()
//     }
//     fn fn_table<'a>(&'a self) -> halobar_config::fmt::FnTable<Self::Data, 1> {
//         FnTable([
//             ("icon", |data| Some(data.state.state_icon().to_string())),
//             ("up_speed", |data| Some(format!("{}", data.up_speed))),
//             ("down_speed", |data| Some(format!("{}", data.down_speed))),
//         ])
//     }
//     fn segments<'s>(&'s self) -> FmtSegments<'s> {
//         if self.data.is_online {
//             self.format
//         } else {
//             self.format_offline
//         }
//     }
//     fn set_data(&mut self, data: Self::Data) {
//         self.data = data;
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
    #[error("Failed to join task")]
    JoinError,
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
impl BackendModule for Network {
    type Error = NetError;
    type Input = (NetKnown, SystemConnection);
    async fn run<'r, D: Into<DisplayOutput>>(
        runtime: Arc<Runtime>,
        input: Self::Input,
        yield_sender: Arc<mpsc::Sender<ModuleType<D>>>,
    ) -> Result<bool, Self::Error> {
        trace!("Starting module");

        let (config, conn) = input;

        if !config.is_valid() {
            return Err(NetError::NetDisabled);
        }

        let conn = conn.0;

        let network_manager = network_manager::NetworkManagerProxy::builder(&conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let mut primary_path = network_manager.primary_connection().await?;

        let mut primary_stream = network_manager.receive_primary_connection_changed().await;

        let (s, mut property_receiver) = mpsc::channel(8);
        let property_sender = Arc::new(s);

        let (kill_sender, r) = flume::unbounded();
        let kill_receiver = Arc::new(r);

        let (output_sender, output_receiver) = BiChannel::new(
            6,
            Some("Networkmanager module"),
            Some("Networkmanager receiver"),
        );
        yield_sender
            .send(ModuleType::Loop(output_receiver))
            .await
            .map_err(|e| {
                error!("Failed to send message to agggregator: {e}");
                NetError::SendError
            })?;

        let format_task = runtime.spawn(async move {
            while let Some(prop) = property_receiver.recv().await {
                debug!("Format task received prop: {}", prop);
            }
        });

        let mut current_listener: Option<tokio::task::JoinHandle<Result<(), NetError>>> =
            Some(tokio::spawn(active_conn_listen(
                primary_path.clone(),
                conn.clone(),
                Arc::clone(&kill_receiver),
                Arc::clone(&property_sender),
            )));

        // I split this into its own future because I want to be able to run cleanup code after it throws errors at me
        let primary_stream_listener = async {
            while let Some(maybe_path) = primary_stream.next().await {
                // Send the kill error before getting the current connection path, because then I don't have
                // to worry about weird mut references and whatnot.
                kill_sender.send_async(()).await.map_err(|e| {
                    error!("Failed to send kill signal to networkmanager modules: {e}");
                    NetError::SendError
                })?;

                // it might not have quit fully when I sent the kill signal for some reason.
                if let Some(ref mut handle) = current_listener {
                    match handle.await {
                        Ok(Ok(())) => {}
                        Ok(Err(e)) => return Err(e),
                        Err(e) => {
                            error!("Error joining task: {e}");
                            return Err(NetError::JoinError);
                        }
                    };
                }
                primary_path = maybe_path.get().await?;

                current_listener.replace(tokio::spawn(active_conn_listen(
                    primary_path,
                    conn.clone(),
                    Arc::clone(&kill_receiver),
                    Arc::clone(&property_sender),
                )));
            }

            Ok::<(), NetError>(())
        };

        let run_result = primary_stream_listener.await;

        if !format_task.is_finished() {
            format_task.abort();
        }

        // release the error after cleanups
        run_result?;

        Ok(false)
    }
}

/// Listen to interfaces on the active connection
async fn active_conn_listen<'c>(
    primary_path: OwnedObjectPath,
    conn: zbus::Connection,
    kill_receiver: Arc<flume::Receiver<()>>,
    property_sender: Arc<mpsc::Sender<PropertyType>>,
) -> Result<(), NetError> {
    trace!("Creating ActiveProxy from path '{primary_path}'");

    let active_proxy = xmlgen::active_connection::ActiveProxy::builder(&conn)
        .path(primary_path)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    let devices = active_proxy.devices().await?;

    // There should just be one device
    // Safety: Having no active devices listening to the active connection should be impossible.
    let device_path = devices
        .into_iter()
        .next()
        .expect("Networkmanager failed to detect any devices for active connection!");

    let stats_proxy = xmlgen::device::StatisticsProxy::builder(&conn)
        .path(device_path)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    macro_rules! listen {
        ($( $proxy:expr => $( $prop:tt ),+ );+) => {
            async {
                ::tokio::try_join!( $($( async {
                    property_sender.send($proxy.$prop().await?.into()).await.map_err(|e| {
                        error!("Failed to send current '{}' to format receiver: {e}", stringify!($prop));
                        NetError::SendError
                    })
                } ),+),+ )?;

                $($(
                    ::paste::paste! {
                        let mut [<$proxy _ $prop _stream>] = $proxy.[<receive_ $prop _changed>]().await;
                    }
                )+)+

                loop {
                    select! {
                        biased;
                        res = kill_receiver.recv_async() => {
                            res.map_err(|e| {
                                error!("Shutdown receiver returned an error: {e}");
                                NetError::RecvError
                            })?;
                            break;
                        }
                        $($(
                            Some(s) = ::paste::paste! {[<$proxy _ $prop _stream>]}.next() => {
                                let raw_value = s.get().await?;
                                let prop = raw_value.into();
                                if let Err(e) = property_sender.send(prop).await {
                                    error!("Failed to send {}::{} to receiver: {e}", stringify!($proxy), stringify!($prop));
                                    return Err(NetError::SendError)
                                }
                            }
                        )+)+
                    }
                }
                Ok::<(), NetError>(())
            }
        };
    }

    let listeners_future = listen![active_proxy => state; stats_proxy => rx_bytes, tx_bytes];

    listeners_future.await?;

    Ok::<(), NetError>(())
}

/// TODO: Formatter struct with format args listening for these in a task
#[derive(Debug, strum_macros::Display, derive_more::From)]
enum PropertyType {
    UpSpeed(variants::UpSpeed),
    DownSpeed(variants::DownSpeed),
    Ssid(variants::Ssid),
    IfaceName(String),
    ActiveConnectionState(variants::NMActiveConnectionState),
}
