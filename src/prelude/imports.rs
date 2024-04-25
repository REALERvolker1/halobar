pub(crate) use color_eyre::{
    eyre::{bail, eyre},
    Report,
};
pub(crate) use halobar_config::{
    config_struct,
    fmt::{FmtSegmentVec, FmtSegments, FormatStr, HaloFormatter},
};
pub(crate) use halogen::{Event, Message};
pub(crate) use once_cell::sync::{Lazy, OnceCell};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use smart_default::SmartDefault;
pub(crate) use std::{
    cell::{Cell, RefCell},
    env, io,
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
    sync::Arc,
    time::Duration,
};
pub(crate) use strum::{EnumMessage, VariantArray, VariantNames};
pub(crate) use tokio::{
    fs, join, select,
    sync::{
        broadcast,
        mpsc::{self, Receiver, Sender, UnboundedReceiver, UnboundedSender},
        Mutex, RwLock, Semaphore,
    },
    try_join,
};
pub(crate) use tracing::{debug, error, info, instrument, trace, warn};
