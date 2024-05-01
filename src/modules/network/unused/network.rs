impl Network {
    pub async fn init(
        config: NetKnown,
        conn: zbus::Connection,
        sender: oneshot::Sender<BiChannel<Event, NetData>>,
    ) -> Result<(), NetError> {
        if !config.is_valid() {
            return Err(NetError::NetDisabled);
        }

        let network_manager = network_manager::NetworkManagerProxy::builder(&conn)
            .build()
            .await?;

        let primary = network_manager.primary_connection().await?;

        let mut primary_stream = network_manager.receive_primary_connection_changed().await;

        let (s, r) = flume::bounded(1);
        let sender = Arc::new(s);
        let receiver = Arc::new(r);

        while let Some(conn) = primary_stream.next().await {
            let conn_path = conn.get().await?;

            primary_stream
        }

        // let mut connection_query = {
        //     let query = {

        //     } else {
        //         chosen_connection(&config.interface, &network_manager, &conn).await
        //     };

        //     match query {
        //         Ok(t) => Some(t),
        //         Err(e) => match e {
        //             NetError::NetDisabled => None,
        //             _ => return Err(e),
        //         },
        //     }
        // };

        // if config.interface.is_empty() {} else {
        //     let mut devices_stream = network_manager.receive_devices_changed().await;

        //     while let Some(device) = devices_stream.next().await {
        //         let devices = device.get().await?;
        //         if devices.is_empty() || !devices.contains(x)

        //         connection_query = ;

        //         if let Some((active, device)) = connection_query {
        //         } else {
        //             let mut state_stream = network_manager.receive_state_changed().await;

        //             while let Some(state) = state_stream.next().await {
        //                 let current = state.get().await?;
        //                 debug!("Networkmanager state: {current}");
        //             }
        //         }
        //     }
        // }

        Ok(())
    }
}

/// Autodetection
async fn primary_connection<'a>(
    network_manager: &network_manager::NetworkManagerProxy<'a>,
    conn: &'a zbus::Connection,
) -> Result<
    (
        xmlgen::active_connection::ActiveProxy<'a>,
        xmlgen::device::DeviceProxy<'a>,
    ),
    NetError,
> {
    let active_path = network_manager.primary_connection().await?;
    debug!("Active networkmanager connection: {active_path}");

    let active = xmlgen::active_connection::ActiveProxy::builder(conn)
        .path(active_path)?
        .build()
        .await?;
    let active_devices = active.devices().await?;

    let mut device = None;

    for path in active_devices {
        let proxy = xmlgen::device::DeviceProxy::builder(&conn)
            .path(path)?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let ty = proxy.device_type().await?;

        match ty {
            NMDeviceType::Dummy | NMDeviceType::Loopback => {
                trace!("Device has invalid network type, skipping...");
                continue;
            }
            _ => {
                device.replace(proxy);
                break;
            }
        }
    }

    // How do you have a connection active with no devices??

    // Answer: If it is a loopback device. This should always panic when the active connection is a loopback device.
    // TODO: Remove this unused code
    let device = device.expect(
        "No devices associated with active networkmanager connection! Please file a bug report!",
    );

    Ok((active, device))
}

/// This is split into a different function so I can easily retry connecting.
async fn get_connected<'a>(
    network_manager: &network_manager::NetworkManagerProxy<'a>,
    conn: &'a zbus::Connection,
    interface: Option<&str>,
) -> Result<
    Option<(
        xmlgen::device::DeviceProxy<'a>,
        xmlgen::active_connection::ActiveProxy<'a>,
    )>,
    NetError,
> {
    let active_devices = network_manager.devices().await?;
    if active_devices.is_empty() {
        return Ok(None);
    }
    let mut device_proxy = None;

    for device in active_devices.into_iter() {
        let proxy = xmlgen::device::DeviceProxy::builder(&conn)
            .path(device)?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        match interface.as_ref() {
            Some(name) => {
                let iface = match proxy.interface().await {
                    Ok(i) => i,
                    Err(e) => {
                        warn!("Error getting proxy interface: {e}");
                        continue;
                    }
                };

                if name == &iface {
                    device_proxy.replace(proxy);
                    break;
                }
            }
            None => {
                let device_type = match proxy.device_type().await {
                    Ok(d) => d,
                    Err(e) => {
                        warn!("Error getting proxy device type: {e}");
                        continue;
                    }
                };

                // This is very naive. the user should ideally have a wifi or ethernet connection in the
                // majority of cases, I can't think of a better way to do this without bloating things to hell.
                match device_type {
                    NMDeviceType::Ethernet | NMDeviceType::Wifi | NMDeviceType::WIFI_P2P => {
                        device_proxy.replace(proxy);
                        break;
                    }
                    _ => {
                        warn!("Invalid device type: {device_type}, skipping");
                    }
                }
            }
        }
    }

    let device = match device_proxy {
        Some(d) => d,
        None => {
            return Err(NetError::InvalidInterface(
                interface.unwrap_or("None").to_owned(),
            ))
        }
    };

    // I got this from the list of active devices. It should just work.
    let active_connection_path = match device.active_connection().await {
        Ok(c) => c,
        Err(e) => {
            warn!("Could not get the active networkmanager connection: {e}");
            return Ok(None);
        }
    };
    let active_connection = xmlgen::active_connection::ActiveProxy::builder(&conn)
        .path(active_connection_path)?
        .build()
        .await?;

    Ok(Some((device, active_connection)))
}
