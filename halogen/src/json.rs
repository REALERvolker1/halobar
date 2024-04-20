use serde::de::DeserializeOwned;

// #[inline]
// pub fn to_string_pretty<S: ?Sized + Serialize>(value: &S) -> Result<String, serde_json::Error> {
//     serde_json::to_string_pretty(value)
// }

#[cfg(all(test, feature = "serde_json"))]
pub use serde_json::to_string_pretty;
#[cfg(feature = "serde_json")]
pub use serde_json::Error;

#[cfg(all(test, feature = "simd-json"))]
pub use simd_json::to_string_pretty;
#[cfg(feature = "simd-json")]
pub use simd_json::Error;

#[cfg(feature = "serde_json")]
#[inline]
pub fn from_string<D: DeserializeOwned>(input_string: &str) -> Result<D, Error> {
    serde_json::from_str(input_string)
}

#[cfg(feature = "simd-json")]
#[inline]
pub fn from_string<D: DeserializeOwned>(input_string: &str) -> Result<D, Error> {
    let mut owned_string = input_string.to_owned();
    unsafe { simd_json::from_str(owned_string.as_mut_str()) }
}