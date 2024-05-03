macro_rules! zbus_conn {
    ($($conn_type:ident: $conn_fn:tt),+) => {
        ::paste::paste! {$(
            #[doc = "A [`zbus::Connection`] that contains a connection to the " $conn_fn " bus"]
            #[derive(Debug, Clone, derive_more::AsRef)]
            pub struct $conn_type(pub zbus::Connection);
            impl $conn_type {
                #[doc = "Create a new zbus connection that is specifically connected to the " $conn_fn " bus."]
                #[inline]
                pub async fn new() -> zbus::Result<Self> {
                    let conn = zbus::Connection::$conn_fn().await?;
                    Ok(Self(conn))
                }
            }
        )+}
    };
}
zbus_conn![SystemConnection: system, SessionConnection: session];
