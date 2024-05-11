use once_cell::sync::OnceCell;
use zbus::Connection;

macro_rules! zbus_conn {
    ($( $kind:tt ),+) => {
        paste::paste! {$(
            static [<ZBUS_ $kind:upper>]: OnceCell<zbus::Result<Connection>> = OnceCell::new();

            #[doc = "Get the current `" $kind "` zbus connection"]
            pub async fn [<get_zbus_ $kind>]() -> zbus::Result<Connection> {
                if let Some(conn) = [<ZBUS_ $kind:upper>].get() {
                    return conn.clone();
                }

                let connection = Connection::$kind().await;

                // Safety: I know it is unset
                [<ZBUS_ $kind:upper>].set(connection.clone()).unwrap();

                connection
            }
        )+}
    };
}
zbus_conn![system, session];

// use crate::{modules::{network::NetData, BiChannel}, prelude::*};

// pub struct SysInfo {
//     system: sysinfo::System,
//     channel: BiChannel<SysInfoRefreshDiscriminants, SysInfoRefresh>,
// }

// /// A message that you send to [`SysInfo`] to make it refresh information
// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
// pub enum SysInfoRefresh {
//     Network(NetData),
//     Cpu,
//     Memory,
//     Disks,
// }
