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
    panic!("g");

    let config_flags = NMPropertyFlags {
        up_speed: true,
        down_speed: true,
        ssid: true,
        iface_name: true,
        strength: true,
        state: true,
        mode: true,
    };

    Ok(())
}
