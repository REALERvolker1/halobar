use std::ops::Deref;

use super::*;

macro_rules! zvariant {
    ($type:ty => $enum:ty) => {
        impl From<::zbus::zvariant::OwnedValue> for $enum {
            fn from(value: ::zbus::zvariant::OwnedValue) -> Self {
                match value.downcast_ref::<$type>() {
                    Ok(v) => Self::from_repr(v).unwrap_or_default(),
                    Err(e) => {
                        error!(
                            "Failed to convert zvariant value into {}: {e}",
                            stringify!($enum),
                        );
                        Self::default()
                    }
                }
            }
        }
    };
}

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
zvariant!(u32 => BatteryState);

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
zvariant!(u32 => WarningLevel);

/// Source: https://upower.freedesktop.org/docs/Device.html
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
pub enum DeviceType {
    #[default]
    Unknown = 0,
    LinePower = 1,
    /// If the value is set to "Battery", you will need to verify that the property `power-supply`
    /// has the value "true" before considering it as a laptop battery.
    ///
    /// Otherwise it will likely be the battery for a device of an unknown type.
    Battery = 2,
    Ups = 3,
    Monitor = 4,
    Mouse = 5,
    Keyboard = 6,
    Pda = 7,
    Phone = 8,
    MediaPlayer = 9,
    Tablet = 10,
    Computer = 11,
    GamingInput = 12,
    Pen = 13,
    Touchpad = 14,
    Modem = 15,
    Network = 16,
    Headset = 17,
    Speakers = 18,
    Headphones = 19,
    Video = 20,
    OtherAudio = 21,
    RemoteControl = 22,
    Printer = 23,
    Scanner = 24,
    Camera = 25,
    Wearable = 26,
    Toy = 27,
    BluetoothGeneric = 28,
}
zvariant!(u32 => DeviceType);

/// For some asinine reason, UPower returns a String
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
    Deserialize,
    Serialize,
)]
pub enum CriticalAction {
    #[default]
    Unknown,
    HybridSleep,
    Hibernate,
    PowerOff,
}
impl TryFrom<::zbus::zvariant::OwnedValue> for CriticalAction {
    type Error = zvariant::Error;
    fn try_from(value: ::zbus::zvariant::OwnedValue) -> Result<Self, Self::Error> {
        let value_string: String = value.try_into()?;

        let me = Self::from_str(&value_string).unwrap_or_default();
        Ok(me)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, derive_more::AsRef,
)]
pub struct Percentage(u8);
impl Percentage {
    pub fn get(&self) -> u8 {
        self.0
    }
    /// Tries to make a new percentage. Returns the input as an error if the int was invalid.
    pub fn try_new(input: u8) -> Result<Self, u8> {
        if input > 100 {
            return Err(input);
        }

        Ok(Self(input))
    }
}
impl TryFrom<::zbus::zvariant::OwnedValue> for Percentage {
    type Error = ::zbus::zvariant::Error;
    fn try_from(value: ::zbus::zvariant::OwnedValue) -> Result<Self, Self::Error> {
        let tried = match value.deref() {
            Value::I32(i) => Self::try_new(i.unsigned_abs() as u8),
            Value::I16(i) => Self::try_new(i.unsigned_abs() as u8),
            Value::I64(i) => Self::try_new(i.unsigned_abs() as u8),
            Value::U8(i) => Self::try_new(*i),
            Value::U16(i) => Self::try_new(*i as u8),
            Value::U32(i) => Self::try_new(*i as u8),
            Value::U64(i) => Self::try_new(*i as u8),

            Value::F64(f) => Self::try_new(f.round() as u8),
            _ => Err(0),
        };

        tried.map_err(|_| {
            trace!("Failed to convert value {value:?} to Percentage");
            zvariant::Error::IncorrectType
        })
    }
}

pub const BATTERY_ICONS_CHARGING: [char; 10] = ['󰢟', '󰢜', '󰂆', '󰂇', '󰂈', '󰢝', '󰂉', '󰢞', '󰂊', '󰂋'];

pub const BATTERY_ICONS_DISCHARGING: [char; 10] =
    ['󰂎', '󰁺', '󰁻', '󰁼', '󰁽', '󰁾', '󰁿', '󰂀', '󰂁', '󰂂'];

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, derive_more::From)]
pub struct Energy(f64);
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, derive_more::From)]
pub struct EnergyRate(f64);
