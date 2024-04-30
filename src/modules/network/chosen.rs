use super::*;
use futures_util::TryStreamExt;
use xmlgen::{
    active_connection::ActiveProxy, device::DeviceProxy, network_manager::NetworkManagerProxy,
};
use zbus::{zvariant::OwnedObjectPath, Connection};

enum FindDeviceOption<'a> {
    Device(DeviceProxy<'a>, OwnedObjectPath),
    Invalid(OwnedObjectPath),
}

/// Find the device that the user chose
pub async fn find_device<'a>(
    conn: &'a Connection,
    netman: &'a NetworkManagerProxy<'a>,
    name: &str,
) -> Result<(DeviceProxy<'a>, OwnedObjectPath), NetError> {
    let devices = netman.devices().await?;

    let mut queries = devices
        .into_iter()
        .map(|path| async move {
            let proxy = DeviceProxy::builder(conn)
                .cache_properties(CacheProperties::No)
                .path(path.clone())?
                .build()
                .await?;

            let proxy_device = proxy.interface().await?;

            if proxy_device == name {
                return Ok(FindDeviceOption::Device(proxy, path));
            }

            Ok::<FindDeviceOption<'a>, zbus::Error>(FindDeviceOption::Invalid(path))
        })
        .collect::<FuturesUnordered<_>>();

    while let Some(query) = queries.try_next().await? {
        match query {
            FindDeviceOption::Device(d, p) => {
                debug!("Networkmanager found device {name} at '{p}'");
                return Ok((d, p));
            }
            FindDeviceOption::Invalid(p) => {
                debug!("Networkmanager skipping device at path '{p}'");
            }
        }
    }

    Err(NetError::InvalidInterface(name.to_owned()))
}

pub async fn get_active_connection<'a>(
    conn: &'a Connection,
    device: &'a DeviceProxy<'a>,
) -> Result<ActiveProxy<'a>, NetError> {
    let path = device.active_connection().await.map_err(|e| {
        debug!("Error getting active networkmanager connection: {e}");
        // The only logical conclusion
        NetError::NetDisabled
    })?;

    let proxy = ActiveProxy::builder(conn)
        .cache_properties(CacheProperties::Lazily)
        .path(path)?
        .build()
        .await?;

    Ok(proxy)
}

pub async fn run<'a>(
    conn: &'a Connection,
    netman: &'a NetworkManagerProxy<'a>,
    name: &'a str,
) -> Result<(), NetError> {
    let (device_proxy, device_path) = find_device(conn, netman, name).await?;

    let active_proxy = RwLock::new(get_active_connection(conn, &device_proxy).await?);

    let (s, mut interrupt_receiver) = mpsc::channel(1);
    let interrupt_sender = Arc::new(s);

    let state_watch = async {
        loop {
            let mut state_stream = {
                let proxy = active_proxy.read().await;
                proxy.receive_state_changed()
            }
            .await;

            loop {
                select! {
                    Some(_) = interrupt_receiver.recv() => {
                        break;
                    }
                    Some(state) = state_stream.next() => {
                        match state.get().await {
                            Ok(state) => {
                                debug!("Networkmanager state changed: {state}");
                            }
                            Err(e) => {
                                error!("Error getting networkmanager state: {e}");
                            }
                        }
                    }
                }
            }
        }
    };

    let active_watch = async {
        let mut active_stream = device_proxy.receive_active_connection_changed().await;

        while let Some(active_path) = active_stream.next().await {
            let active = match active_path.get().await {
                Ok(p) => {
                    debug!("Networkmanager Active connection path changed: {p}");
                    match ActiveProxy::builder(conn).path(p) {
                        Ok(b) => match b.build().await {
                            Ok(p) => p,
                            Err(e) => {
                                warn!("Failed to build a proxy for the active network: {e}");
                                continue;
                            }
                        },
                        Err(e) => {
                            warn!(
                                "Failed to build a proxy for the active network with the required path: {e}"
                            );
                            continue;
                        }
                    }
                }
                Err(e) => {
                    warn!("Networkmanager failed to get active path from event: {e}");
                    continue;
                }
            };

            {
                let mut lock = active_proxy.write().await;
                *lock = active;
            }

            interrupt_sender.send(()).await.expect(
                "Failed to send internal message in networkmanager. Please file a bug report!",
            );
        }
    };

    join!(state_watch, active_watch);

    Ok(())
}
