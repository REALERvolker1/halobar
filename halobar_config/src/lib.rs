#[cfg(test)]
mod test;

mod config_struct;

use std::{env, path::PathBuf};

/// The environment variable to query for the XDG config home
const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";

/// Directory stubs to append to the HOME environment variable. This is an array because the standard might change in the future
const CONFIG_JOIN_STUBS: [&str; 1] = [".config"];

/// The environment variable to query for the user home directory
const HOME: &str = "HOME";

/// Returns XDG_CONFIG_HOME without checking if it exists or is valid.
///
/// Will return None if both the XDG_CONFIG_HOME and the HOME environment variables are unset.
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
