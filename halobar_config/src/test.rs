use crate::*;

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
