use super::*;

/// The current state of the battery, an enum based on its representation in upower
///
/// For upower, this is well-defined. For sysfs, check out `/usr/lib/modules/<kernel>/build/include/linux/power_supply.h`
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Default,
    strum_macros::Display,
    strum_macros::FromRepr,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
    zvariant::Type,
    Deserialize_repr,
    Serialize_repr,
)]
#[repr(u32)]
#[strum(ascii_case_insensitive, serialize_all = "kebab-case")]
pub enum BatteryState {
    #[default]
    Unknown = 0,
    Charging = 1,
    Discharging = 2,
    Empty = 3,
    FullyCharged = 4,
    PendingCharge = 5,
    PendingDischarge = 6,
}
impl From<zvariant::OwnedValue> for BatteryState {
    fn from(value: zvariant::OwnedValue) -> Self {
        match value.downcast_ref::<u32>() {
            Ok(v) => Self::from_repr(v).unwrap_or_default(),
            Err(e) => {
                error!("Failed to convert zvariant value into BatteryState: {}", e);
                Self::default()
            }
        }
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Default,
    strum_macros::Display,
    strum_macros::FromRepr,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
    zvariant::Type,
    Deserialize_repr,
    Serialize_repr,
)]
#[repr(u32)]
#[strum(ascii_case_insensitive, serialize_all = "kebab-case")]
pub enum WarningLevel {
    #[default]
    Unknown = 0,
    None = 1,
    /// Only for UPSes
    Discharging = 2,
    Low = 3,
    Critical = 4,
    /// When the upower battery action runs (on my system it shuts down)
    Action = 5,
}
impl From<zvariant::OwnedValue> for WarningLevel {
    fn from(value: zvariant::OwnedValue) -> Self {
        match value.downcast_ref::<u32>() {
            Ok(v) => Self::from_repr(v).unwrap_or_default(),
            Err(e) => {
                error!("Failed to convert zvariant value into WarningLevel: {}", e);
                Self::default()
            }
        }
    }
}

pub const BATTERY_ICONS_CHARGING: [char; 10] = ['󰢟', '󰢜', '󰂆', '󰂇', '󰂈', '󰢝', '󰂉', '󰢞', '󰂊', '󰂋'];

pub const BATTERY_ICONS_DISCHARGING: [char; 10] =
    ['󰂎', '󰁺', '󰁻', '󰁼', '󰁽', '󰁾', '󰁿', '󰂀', '󰂁', '󰂂'];
