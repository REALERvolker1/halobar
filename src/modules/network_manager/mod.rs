mod listener;
mod props;
mod proxy_functions;
mod speed;
mod variants;
mod xmlgen;

pub use props::*;
pub use variants::NMActiveConnectionState;

pub async fn live_test() -> NetResult<()> {
    let conn = super::SystemConnection::new().await?;

    let config = NetKnown::default();

    let config_flags = NMPropertyFlags::from_segments(config.format.segments());

    let mut module = listener::NetModule::new(&conn.0, config).await?;

    module.run().await?;

    Ok(())
}
