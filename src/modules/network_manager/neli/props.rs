use super::*;

#[derive(Debug, Default)]
pub(super) struct Speed {
    pub rx_total: u64,
    pub tx_total: u64,

    pub rx_per_second: u64,
    pub tx_per_second: u64,

    pub last_checked: Instant,
}
impl Speed {
    #[instrument(level = "debug", skip(socket))]
    pub async fn new(socket: &mut neli_wifi::AsyncSocket) -> NetResult<Self> {


        let interface = neli::

        Ok(Self {
            rx_total: 0,
            tx_total: 0,
            rx_per_second: 0,
            tx_per_second: 0,
            last_checked: Instant::now(),
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
