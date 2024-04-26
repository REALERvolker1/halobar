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
