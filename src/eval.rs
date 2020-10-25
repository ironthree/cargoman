use parse_cfg::{Cfg, Target};

// https://doc.rust-lang.org/reference/conditional-compilation.html
pub fn eval_target_cfg(cfg: &Cfg) -> Result<bool, String> {
    match cfg {
        Cfg::Any(cfgs) => Ok(cfgs.iter().map(|cfg| eval_target_cfg(cfg)).any(|eval| match eval {
            Ok(result) => result,
            Err(error) => {
                log::error!("{}", error);
                false
            },
        })),
        Cfg::All(cfgs) => Ok(cfgs.iter().map(|cfg| eval_target_cfg(cfg)).all(|eval| match eval {
            Ok(result) => result,
            Err(error) => {
                log::error!("{}", error);
                false
            },
        })),
        Cfg::Not(cfg) => Ok(!eval_target_cfg(cfg)?),
        Cfg::Equal(a, b) => match a.as_str() {
            "target_arch" => Ok(b != "wasm32"),
            // ignore the "target_endian" flag
            "target_endian" => Ok(true),
            "target_env" => Ok(vec!["", "gnu"].contains(&b.as_str())),
            "target_family" => Ok(b == "unix"),
            // ignore the "target_feature" flag
            "target_feature" => Ok(true),
            // these are all documented target_os values except "linux"
            "target_os" => Ok(!vec![
                "windows",
                "macos",
                "ios",
                "android",
                "freebsd",
                "dragonfly",
                "openbsd",
                "netbsd",
                "emscripten",
            ]
            .contains(&b.as_str())),
            // ignore the "target_pointer_width" flag
            "target_pointer_width" => Ok(true),
            "target_vendor" => Ok(!vec!["apple", "fortanix", "pc"].contains(&b.as_str())),
            _ => Err(format!("Unrecognised target flag: {}", a)),
        },
        Cfg::Is(string) => match string.as_str() {
            // assume release mode
            "debug_assertions" => Ok(false),
            // assume tests are enabled
            "test" => Ok(true),
            // assume proc_macro mode is enabled
            "proc_macro" => Ok(true),
            "unix" => Ok(true),
            "windows" => Ok(false),
            _ => Err(format!("Unrecognised target option: {}", string)),
        },
    }
}

pub fn is_linux_target(target: &str) -> Result<bool, String> {
    let cfg = parse_cfg::parse_target(target).map_err(|err| format!("Failed to parse target: {}", err))?;

    match cfg {
        Target::Triple {
            arch: _arch,
            vendor,
            os,
            env: _env,
        } => {
            // are those two conditions sufficient?
            Ok(vendor == "unknown" && os == "linux")
        },
        Target::Cfg(cfg) => eval_target_cfg(&cfg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix() {
        let string = "cfg(unix)";
        assert_eq!(is_linux_target(string).unwrap(), true);
    }

    #[test]
    fn windows() {
        let string = "cfg(windows)";
        assert_eq!(is_linux_target(string).unwrap(), false);
    }

    #[test]
    fn cfg_target_os_macos() {
        let string = "cfg(target_os = \"macos\")";
        assert_eq!(is_linux_target(string).unwrap(), false);
    }

    #[test]
    fn cfg_target_arch_wasm32() {
        let string = "cfg(target_arch = \"wasm32\")";
        assert_eq!(is_linux_target(string).unwrap(), false);
    }

    #[test]
    fn cfg_not_target_arch_wasm32() {
        let string = "cfg(not(target_arch = \"wasm32\"))";
        assert_eq!(is_linux_target(string).unwrap(), true);
    }

    #[test]
    fn cfg_all_target_arch_wasm32_not_target_os_emscripten() {
        let string = "cfg(all(target_arch = \"wasm32\", not(target_os = \"emscripten\")))";
        assert_eq!(is_linux_target(string).unwrap(), false);
    }
}
