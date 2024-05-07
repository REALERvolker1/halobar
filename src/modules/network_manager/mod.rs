mod data_functions;
mod listener;
mod props;
mod variants;
mod xmlgen;

pub use props::*;
pub use variants::NMActiveConnectionState;

use crate::prelude::*;

pub async fn live_test() -> NetResult<()> {
    let conn = SystemConnection::new().await?;

    let config = NetKnown::default();
    let config_flags = NMPropertyFlags::from_segments(config.format.segments());

    let proxies = data_functions::NetworkProxies::new(&conn.0, config_flags, None).await?;

    Ok(())
}
