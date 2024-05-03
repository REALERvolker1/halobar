mod xmlgen;

// mod chosen;
mod variants;

use futures_util::TryFutureExt;
use zbus::{
    proxy::{CacheProperties, PropertyStream},
    zvariant::OwnedObjectPath,
};

use self::{
    variants::{NMDeviceType, NMState},
    xmlgen::network_manager,
};

use super::*;

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

struct FormatNet {
    /// TODO: Allow choosing decimal rounding
    format: FmtSegmentVec,
    format_offline: FmtSegmentVec,

    state: Cell<FormatState>,
}
impl FormatNet {
    pub async fn format_task(
        self,
        output_sender: mpsc::Sender<String>,
        mut format_receiver: mpsc::Receiver<PropertyType>,
    ) -> ! {
        while let Some(prop) = format_receiver.recv().await {}
        unreachable!("Format task closed unexpectedly!");
    }
}

config_struct! {
    [Net]
    show_speed_up: bool = true,
    show_speed_down: bool = true,
    show_ssid: bool = true,
    show_device: bool = true,
    show_state: bool = true,
    // format: FormatStr = FormatStr::default(),
    poll_rate_seconds: u64 = 5,
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
    const MODULE_REQUIREMENTS: &'static [ModuleRequirementDiscriminants] =
        &[ModuleRequirementDiscriminants::SystemDbus];
    const MODULE_TYPE: ModuleType = ModuleType::Network;
    // #[inline]
    // fn output_type(&self) -> OutputTypeDiscriminants {
    //     OutputTypeDiscriminants::Loop
    // }
    async fn run(
        runtime: Arc<Runtime>,
        input: Self::Input,
        yield_sender: Arc<mpsc::UnboundedSender<(OutputType, ModuleType)>>,
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

        let primary_path = network_manager.primary_connection().await?;

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
            .send((OutputType::Loop(output_receiver), Self::MODULE_TYPE))
            .map_err(|e| {
                error!("Failed to send message to agggregator: {e}");
                NetError::SendError
            })?;

        let format_task = runtime.spawn(async move {
            while let Some(prop) = property_receiver.recv().await {
                debug!("Format task received prop: {:?}", prop);
            }
        });

        let config = Arc::new(config);

        let mut current_listener: Option<tokio::task::JoinHandle<Result<(), NetError>>> =
            Some(tokio::spawn(active_conn_listen(
                primary_path.clone(),
                conn.clone(),
                Arc::clone(&kill_receiver),
                Arc::clone(&property_sender),
                Arc::clone(&config),
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
                let primary_path = maybe_path.get().await?;

                current_listener.replace(tokio::spawn(active_conn_listen(
                    primary_path,
                    conn.clone(),
                    Arc::clone(&kill_receiver),
                    Arc::clone(&property_sender),
                    Arc::clone(&config),
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
    config: Arc<NetKnown>,
) -> Result<(), NetError> {
    trace!("Creating ActiveProxy from path '{primary_path}'");

    let active_proxy = xmlgen::active_connection::ActiveProxy::builder(&conn)
        .path(primary_path)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    let stats_proxy_future = async {
        let devices = active_proxy.devices().await?;

        let device_path = match devices.into_iter().next() {
            Some(d) => d,
            None => {
                error!("Failed to detect any devices for active connection!");
                return Err(NetError::NetDisabled);
            }
        };

        let proxy = xmlgen::device::StatisticsProxy::builder(&conn)
            .path(device_path)?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        Ok::<_, NetError>(proxy)
    };

    let access_point_future = async {
        let access_path = active_proxy.connection().await?;

        let proxy = xmlgen::access_point::AccessPointProxy::builder(&conn)
            .path(access_path)?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;
        Ok::<_, NetError>(proxy)
    };

    let (stats_proxy, access_point_proxy) =
        tokio::try_join!(stats_proxy_future, access_point_future)?;

    let net_poll_duration = Duration::from_secs(config.poll_rate_seconds);

    // Good luck maintaining this one, nerd
    macro_rules! listen {
        ($( $proxy:expr => $( $prop:tt: $prop_type:ident ),+ );+ $(speed_poll: $speed_poll:expr)?) => {
            async {
                ::tokio::try_join!( $($( async {
                    property_sender.send(PropertyType::$prop_type($proxy.$prop().await?)).await.map_err(|e| {
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
                                let prop = PropertyType::$prop_type(raw_value);
                                if let Err(e) = property_sender.send(prop).await {
                                    error!("Failed to send {}::{} to receiver: {e}", stringify!($proxy), stringify!($prop));
                                    return Err(NetError::SendError)
                                }
                            }
                        )+)+

                        $(
                            res = $speed_poll.await => {
                                res?;
                            }
                        )?
                    }
                }
                Ok::<(), NetError>(())
            }
        };
        (speed_poll: $($poll_fn:tt $prop_type:ident),+) => {
            // The network speed must be polled in timed intervals
            async {
                tokio::try_join!($( async {
                    let mut previous_bytes = 0;

                    let mut last_check = Instant::now();

                    loop {
                        // TODO: Find a better way to do this
                        if kill_receiver.try_recv().is_ok() {
                            break;
                        }
                        let bytes = stats_proxy.$poll_fn().await?;

                        // I measure it this way because it allows me to just keep the time since last check
                        let current_check = Instant::now();
                        let duration = current_check.duration_since(last_check);
                        last_check = current_check;

                        let difference = *bytes - previous_bytes;
                        let duration_seconds = duration.as_secs_f64();
                        let bytes_per_second = difference as f64 / duration_seconds;

                        previous_bytes = *bytes;

                        let send_future =
                            property_sender.send(PropertyType::$prop_type(Size::from_bytes(bytes_per_second)));

                        let (send, _) = tokio::join!(send_future, tokio::time::sleep(net_poll_duration));

                        if let Err(e) = send {
                            error!("Failed to send {} to receiver: {e}", stringify!($prop_type));
                            return Err(NetError::SendError);
                        }
                    }
                    Ok::<(), NetError>(())
                } ),+)
            }
        };
    }

    let speed_polls = listen![speed_poll: tx_bytes UpSpeed, rx_bytes DownSpeed];
    let listeners_future = listen![active_proxy => state: ActiveConnectionState; access_point_proxy => ssid: Ssid, strength: Strength];

    tokio::try_join!(speed_polls, listeners_future)?;

    Ok::<(), NetError>(())
}

/// TODO: Formatter struct with format args listening for these in a task
#[derive(Debug)]
enum PropertyType {
    UpSpeed(Size),
    DownSpeed(Size),
    Ssid(variants::Ssid),
    IfaceName(String),
    Strength(u8),
    ActiveConnectionState(variants::NMActiveConnectionState),
}
