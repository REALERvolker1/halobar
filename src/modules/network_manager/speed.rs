use super::props::*;
use super::variants::*;
use super::xmlgen::device::StatisticsProxy;
use crate::prelude::*;
use zbus::CacheProperties;

#[derive(Debug)]
pub(super) struct Speed<'c> {
    pub rx_total: u64,
    pub tx_total: u64,

    pub sender: Arc<mpsc::Sender<NMPropertyType>>,

    pub poll_rate: Duration,
    pub last_checked: Instant,

    pub proxy: StatisticsProxy<'c>,
}
impl<'c> Speed<'c> {
    #[instrument(level = "debug")]
    pub async fn new(
        conn: &'c SystemConnection,
        device_path: OwnedObjectPath,
        sender: Arc<mpsc::Sender<NMPropertyType>>,
        poll_rate: Duration,
    ) -> NetResult<Self> {
        let proxy = StatisticsProxy::builder(&conn.0)
            .path(device_path)?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let (tx_total, rx_total) = try_join![proxy.tx_bytes(), proxy.rx_bytes()]?;

        Ok(Self {
            rx_total: rx_total.0,
            tx_total: tx_total.0,
            sender,
            poll_rate,
            last_checked: Instant::now(),
            proxy,
        })
    }
    #[instrument(level = "trace")]
    pub async fn refresh(&mut self) -> NetResult<()> {
        let (tx_bytes, rx_bytes) = try_join![self.proxy.tx_bytes(), self.proxy.rx_bytes()]?;
        let checked_at = Instant::now();
        let time_interval = self.last_checked.duration_since(checked_at);
        let time_interval_seconds = time_interval.as_secs_f64();

        macro_rules! diff {
            ($type:tt) => {
                ::paste::paste! {{
                    let diff = self.[<$type _total>] - [<$type _bytes>].0;
                    let bytes_per_second = diff as f64 / time_interval_seconds;

                    bytes_per_second.round().abs() as u64
                }}
            };
        }

        try_join!(
            self.sender.send(NMPropertyType::DownSpeed(diff!(rx))),
            self.sender.send(NMPropertyType::UpSpeed(diff!(tx)))
        )?;

        self.tx_total = tx_bytes.0;
        self.rx_total = rx_bytes.0;
        self.last_checked = checked_at;

        Ok(())
    }
    pub async fn run(mut self, kill_receiver: Arc<flume::Receiver<()>>) -> NetResult<()> {
        loop {
            select! {
                recv = kill_receiver.recv_async() => {
                    return recv.map_err(|e| {
                        error!("Failed to receive kill value from sender: {e}");
                        e.into()
                    });
                }
                Err(e) = self.run_update() => {
                    error!("Error running the update function: {e}");
                    return Err(e);
                }
            }
        }
    }
    #[inline]
    async fn run_update(&mut self) -> NetResult<()> {
        let (_, res) = join!(tokio::time::sleep(self.poll_rate), self.refresh());
        res
    }
}
