use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct LandlockSupport {
    pub supported: bool,
    pub abi: Option<u32>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LandlockApplyReport {
    pub support: LandlockSupport,
    pub attempted: bool,
    pub applied: bool,
    pub rules_added: usize,
    pub reason: Option<String>,
}

pub fn detect_support() -> LandlockSupport {
    #[cfg(target_os = "linux")]
    {
        linux::detect_support()
    }
    #[cfg(not(target_os = "linux"))]
    {
        LandlockSupport {
            supported: false,
            abi: None,
            reason: Some("landlock only supported on Linux".to_string()),
        }
    }
}

pub fn apply_path_allowlists(read_paths: &[String], write_paths: &[String]) -> LandlockApplyReport {
    #[cfg(target_os = "linux")]
    {
        linux::apply_path_allowlists(read_paths, write_paths)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (read_paths, write_paths);
        LandlockApplyReport {
            support: detect_support(),
            attempted: false,
            applied: false,
            rules_added: 0,
            reason: Some("landlock only supported on Linux".to_string()),
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::{LandlockApplyReport, LandlockSupport};
    use std::collections::BTreeMap;
    use std::ffi::CString;
    use std::mem;

    const LANDLOCK_CREATE_RULESET_VERSION: u32 = 1;
    const LANDLOCK_RULE_PATH_BENEATH: u32 = 1;

    const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1 << 0;
    const LANDLOCK_ACCESS_FS_WRITE_FILE: u64 = 1 << 1;
    const LANDLOCK_ACCESS_FS_READ_FILE: u64 = 1 << 2;
    const LANDLOCK_ACCESS_FS_READ_DIR: u64 = 1 << 3;
    const LANDLOCK_ACCESS_FS_REMOVE_DIR: u64 = 1 << 4;
    const LANDLOCK_ACCESS_FS_REMOVE_FILE: u64 = 1 << 5;
    const LANDLOCK_ACCESS_FS_MAKE_CHAR: u64 = 1 << 6;
    const LANDLOCK_ACCESS_FS_MAKE_DIR: u64 = 1 << 7;
    const LANDLOCK_ACCESS_FS_MAKE_REG: u64 = 1 << 8;
    const LANDLOCK_ACCESS_FS_MAKE_SOCK: u64 = 1 << 9;
    const LANDLOCK_ACCESS_FS_MAKE_FIFO: u64 = 1 << 10;
    const LANDLOCK_ACCESS_FS_MAKE_BLOCK: u64 = 1 << 11;
    const LANDLOCK_ACCESS_FS_MAKE_SYM: u64 = 1 << 12;
    const LANDLOCK_ACCESS_FS_REFER: u64 = 1 << 13;
    const LANDLOCK_ACCESS_FS_TRUNCATE: u64 = 1 << 14;

    #[repr(C)]
    struct LandlockRulesetAttr {
        handled_access_fs: u64,
    }

    #[repr(C)]
    struct LandlockPathBeneathAttr {
        allowed_access: u64,
        parent_fd: i32,
    }

    fn abi_supported_access_fs(abi: u32) -> u64 {
        let mut mask = LANDLOCK_ACCESS_FS_EXECUTE
            | LANDLOCK_ACCESS_FS_WRITE_FILE
            | LANDLOCK_ACCESS_FS_READ_FILE
            | LANDLOCK_ACCESS_FS_READ_DIR
            | LANDLOCK_ACCESS_FS_REMOVE_DIR
            | LANDLOCK_ACCESS_FS_REMOVE_FILE
            | LANDLOCK_ACCESS_FS_MAKE_CHAR
            | LANDLOCK_ACCESS_FS_MAKE_DIR
            | LANDLOCK_ACCESS_FS_MAKE_REG
            | LANDLOCK_ACCESS_FS_MAKE_SOCK
            | LANDLOCK_ACCESS_FS_MAKE_FIFO
            | LANDLOCK_ACCESS_FS_MAKE_BLOCK
            | LANDLOCK_ACCESS_FS_MAKE_SYM;

        if abi >= 2 {
            mask |= LANDLOCK_ACCESS_FS_REFER;
        }
        if abi >= 3 {
            mask |= LANDLOCK_ACCESS_FS_TRUNCATE;
        }
        mask
    }

    fn read_access_mask(abi: u32) -> u64 {
        let supported = abi_supported_access_fs(abi);
        supported
            & (LANDLOCK_ACCESS_FS_EXECUTE
                | LANDLOCK_ACCESS_FS_READ_FILE
                | LANDLOCK_ACCESS_FS_READ_DIR)
    }

    fn write_access_mask(abi: u32) -> u64 {
        let supported = abi_supported_access_fs(abi);
        let mut mask = LANDLOCK_ACCESS_FS_EXECUTE
            | LANDLOCK_ACCESS_FS_READ_FILE
            | LANDLOCK_ACCESS_FS_READ_DIR
            | LANDLOCK_ACCESS_FS_WRITE_FILE
            | LANDLOCK_ACCESS_FS_REMOVE_DIR
            | LANDLOCK_ACCESS_FS_REMOVE_FILE
            | LANDLOCK_ACCESS_FS_MAKE_CHAR
            | LANDLOCK_ACCESS_FS_MAKE_DIR
            | LANDLOCK_ACCESS_FS_MAKE_REG
            | LANDLOCK_ACCESS_FS_MAKE_SOCK
            | LANDLOCK_ACCESS_FS_MAKE_FIFO
            | LANDLOCK_ACCESS_FS_MAKE_BLOCK
            | LANDLOCK_ACCESS_FS_MAKE_SYM;

        if abi >= 2 {
            mask |= LANDLOCK_ACCESS_FS_REFER;
        }
        if abi >= 3 {
            mask |= LANDLOCK_ACCESS_FS_TRUNCATE;
        }
        supported & mask
    }

    pub(super) fn detect_support() -> LandlockSupport {
        match unsafe { landlock_create_ruleset_version() } {
            Ok(abi) => LandlockSupport {
                supported: true,
                abi: Some(abi),
                reason: None,
            },
            Err(err) => LandlockSupport {
                supported: false,
                abi: None,
                reason: Some(err),
            },
        }
    }

    pub(super) fn apply_path_allowlists(
        read_paths: &[String],
        write_paths: &[String],
    ) -> LandlockApplyReport {
        let support = detect_support();
        if !support.supported {
            return LandlockApplyReport {
                support,
                attempted: false,
                applied: false,
                rules_added: 0,
                reason: None,
            };
        }

        let Some(abi) = support.abi else {
            return LandlockApplyReport {
                support,
                attempted: false,
                applied: false,
                rules_added: 0,
                reason: Some("landlock support probe returned no ABI version".to_string()),
            };
        };

        if read_paths.is_empty() {
            return LandlockApplyReport {
                support,
                attempted: false,
                applied: false,
                rules_added: 0,
                reason: Some("landlock allowlists were empty; skipping".to_string()),
            };
        }

        if let Err(code) = prctl::set_no_new_privileges(true) {
            return LandlockApplyReport {
                support,
                attempted: true,
                applied: false,
                rules_added: 0,
                reason: Some(format!("failed to set no_new_privileges: {code}")),
            };
        }

        let mut allowlist: BTreeMap<&str, u64> = BTreeMap::new();
        let read_mask = read_access_mask(abi);
        let write_mask = write_access_mask(abi);

        for path in read_paths {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry = allowlist.entry(trimmed).or_default();
            *entry |= read_mask;
        }

        for path in write_paths {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry = allowlist.entry(trimmed).or_default();
            *entry |= write_mask;
        }

        let handled_access_fs =
            allowlist.values().fold(0u64, |acc, mask| acc | *mask) & abi_supported_access_fs(abi);

        let ruleset_attr = LandlockRulesetAttr { handled_access_fs };

        let ruleset_fd = match unsafe { landlock_create_ruleset(&ruleset_attr) } {
            Ok(fd) => fd,
            Err(err) => {
                return LandlockApplyReport {
                    support,
                    attempted: true,
                    applied: false,
                    rules_added: 0,
                    reason: Some(err),
                };
            }
        };

        let mut rules_added = 0usize;
        for (path, access) in allowlist {
            if access == 0 {
                continue;
            }

            let fd = match open_opath(path) {
                Ok(fd) => fd,
                Err(OpenError::NotFound) => continue,
                Err(OpenError::Other(err)) => {
                    unsafe {
                        libc::close(ruleset_fd);
                    }
                    return LandlockApplyReport {
                        support,
                        attempted: true,
                        applied: false,
                        rules_added,
                        reason: Some(err),
                    };
                }
            };

            let attr = LandlockPathBeneathAttr {
                allowed_access: access,
                parent_fd: fd,
            };

            let added = unsafe { landlock_add_rule(ruleset_fd, &attr) };
            unsafe {
                libc::close(fd);
            }

            if let Err(err) = added {
                unsafe {
                    libc::close(ruleset_fd);
                }
                return LandlockApplyReport {
                    support,
                    attempted: true,
                    applied: false,
                    rules_added,
                    reason: Some(err),
                };
            }

            rules_added += 1;
        }

        match unsafe { landlock_restrict_self(ruleset_fd) } {
            Ok(()) => {
                unsafe {
                    libc::close(ruleset_fd);
                }
                LandlockApplyReport {
                    support,
                    attempted: true,
                    applied: true,
                    rules_added,
                    reason: None,
                }
            }
            Err(err) => {
                unsafe {
                    libc::close(ruleset_fd);
                }
                LandlockApplyReport {
                    support,
                    attempted: true,
                    applied: false,
                    rules_added,
                    reason: Some(err),
                }
            }
        }
    }

    unsafe fn landlock_create_ruleset_version() -> Result<u32, String> {
        let ret = libc::syscall(
            libc::SYS_landlock_create_ruleset,
            std::ptr::null::<libc::c_void>(),
            0usize,
            LANDLOCK_CREATE_RULESET_VERSION,
        );
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(format!("landlock unavailable: {err}"));
        }
        Ok(ret as u32)
    }

    unsafe fn landlock_create_ruleset(attr: &LandlockRulesetAttr) -> Result<i32, String> {
        let ret = libc::syscall(
            libc::SYS_landlock_create_ruleset,
            attr as *const LandlockRulesetAttr,
            mem::size_of::<LandlockRulesetAttr>(),
            0u32,
        );
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(format!("landlock create_ruleset failed: {err}"));
        }
        Ok(ret as i32)
    }

    unsafe fn landlock_add_rule(
        ruleset_fd: i32,
        attr: &LandlockPathBeneathAttr,
    ) -> Result<(), String> {
        let ret = libc::syscall(
            libc::SYS_landlock_add_rule,
            ruleset_fd,
            LANDLOCK_RULE_PATH_BENEATH,
            attr as *const LandlockPathBeneathAttr,
            0u32,
        );
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(format!("landlock add_rule failed: {err}"));
        }
        Ok(())
    }

    unsafe fn landlock_restrict_self(ruleset_fd: i32) -> Result<(), String> {
        let ret = libc::syscall(libc::SYS_landlock_restrict_self, ruleset_fd, 0u32);
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(format!("landlock restrict_self failed: {err}"));
        }
        Ok(())
    }

    enum OpenError {
        NotFound,
        Other(String),
    }

    fn open_opath(path: &str) -> Result<i32, OpenError> {
        let cstr = CString::new(path)
            .map_err(|e| OpenError::Other(format!("invalid path {path:?}: {e}")))?;
        let fd = unsafe { libc::open(cstr.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
        if fd < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::NotFound {
                return Err(OpenError::NotFound);
            }
            return Err(OpenError::Other(format!(
                "failed to open {path:?} for landlock: {err}"
            )));
        }
        Ok(fd)
    }
}
