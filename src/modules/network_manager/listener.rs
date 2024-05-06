use super::*;
use xmlgen::device::{DeviceProxy, StatisticsProxy};
use zbus::{proxy::CacheProperties, zvariant::OwnedObjectPath};

#[derive(Debug)]
pub(super) struct Speed<'c> {
    pub rx_total: u64,
    pub tx_total: u64,

    pub rx_per_second: Size,
    pub tx_per_second: Size,

    pub last_checked: Instant,
    pub proxy: StatisticsProxy<'c>,
}
impl<'c> Speed<'c> {
    #[instrument(level = "debug")]
    pub async fn new(conn: &'c SystemConnection, device_path: OwnedObjectPath) -> NetResult<Self> {
        let proxy = StatisticsProxy::builder(&conn.0)
            .path(device_path)?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let (tx_total, rx_total) = try_join![proxy.tx_bytes(), proxy.rx_bytes()]?;

        Ok(Self {
            rx_total: rx_total.0,
            tx_total: tx_total.0,
            rx_per_second: Size::from_const(0),
            tx_per_second: Size::from_const(0),
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

                    Size::from_bytes(bytes_per_second)
                }}
            };
        }

        self.rx_per_second = diff!(rx);
        self.tx_per_second = diff!(tx);
        self.tx_total = tx_bytes.0;
        self.rx_total = rx_bytes.0;
        self.last_checked = checked_at;

        Ok(())
    }
}
