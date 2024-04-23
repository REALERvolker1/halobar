//! A library that provides helpful functions and macros for programs that need to read config files.

// #![forbid(missing_docs)]

/// Private submodule for the config macro
mod r#macro;

/// Parsing custom formatting strings
pub mod fmt;

use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

/// The environment variable to query for the XDG config home
const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
/// Directory stubs to append to the HOME environment variable. This is an array because the standard might change in the future
const CONFIG_JOIN_STUBS: [&str; 1] = [".config"];
/// The environment variable to query for the user home directory
const HOME: &str = "HOME";

/// Returns XDG_CONFIG_HOME without checking if it exists or is valid.
///
/// Will return None if both the XDG_CONFIG_HOME and the HOME environment variables are unset.
#[tracing::instrument(level = "trace")]
pub fn xdg_config_home() -> Option<PathBuf> {
    if let Some(c) = env::var_os(XDG_CONFIG_HOME) {
        return Some(PathBuf::from(c));
    }

    if let Some(h) = env::var_os(HOME) {
        let mut home = PathBuf::from(h);
        for stub in CONFIG_JOIN_STUBS {
            home.push(stub);
        }
        return Some(home);
    }

    None
}

/// Try to deserialize a file into an object, providing the path.
#[tracing::instrument(level = "debug")]
pub fn try_from_path<D: serde::de::DeserializeOwned>(path: &Path) -> Result<D, Error> {
    let bytes = fs::read(path)?;
    let me = toml_edit::de::from_slice(&bytes)?;

    Ok(me)
}

/// Try to deserialize a file. If that fails, print a message at error level and return the default as Err.
#[tracing::instrument(level = "debug")]
pub fn from_path_or_default<D: serde::de::DeserializeOwned + Default>(
    path: Option<&Path>,
) -> Result<D, D> {
    if let Some(path) = path {
        match try_from_path(path) {
            Ok(c) => return Ok(c),
            Err(e) => tracing::error!("Error reading config file '{}': {}", path.display(), e),
        }
    }

    return Err(D::default());
}

/// Serialize the value into a pretty string
#[inline]
#[tracing::instrument(level = "trace", skip(value))]
pub fn serialized_string<S: serde::Serialize>(value: &S) -> Result<String, toml_edit::ser::Error> {
    toml_edit::ser::to_string_pretty(value)
}

/// The shared error type for halobar_config errors
#[derive(Debug, derive_more::Error, derive_more::Display, derive_more::From)]
pub enum Error {
    /// std::io::Error
    Io(io::Error),
    /// An error deserializing into a struct
    Deserialize(toml_edit::de::Error),
    /// An error that occured while dealing with fmtstrs
    Fmt(fmt::FormatStrError),
}

#[cfg(test)]
mod test {
    use super::*;

    /// This is a fallible test! It relies on environment variables.
    #[test]
    fn config_home() {
        let mut cfg_home = PathBuf::from(env::var_os(HOME).unwrap());
        for stub in CONFIG_JOIN_STUBS {
            cfg_home.push(stub);
        }
        // safety: This is single-threaded
        env::set_var(XDG_CONFIG_HOME, &cfg_home);

        assert_eq!(xdg_config_home().unwrap(), cfg_home);
    }

    config_struct! {
        @config {PartialEq, Eq}
        [TestFile]
        value: u8 = 9,
        is_enabled: bool = true,
        name: String = "Theodore".to_owned(),
    }

    #[test]
    fn read_write_config() {
        // let dir = Path::new(SWP_DIR);
        let dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
        if !dir.is_dir() {
            fs::create_dir_all(&dir).unwrap();
        }
        let path = dir.join("test.toml");

        let my_config = TestFileKnown {
            value: 32,
            is_enabled: false,
            name: "Sharon".into(),
        };
        let my_config_string = serialized_string(&my_config).unwrap();
        fs::write(&path, my_config_string.as_bytes()).unwrap();

        let read = from_path_or_default::<TestFileConfig>(Some(&path)).unwrap();

        fs::remove_file(&path).unwrap();

        assert_eq!(read, my_config.into_wrapped());
    }
}
