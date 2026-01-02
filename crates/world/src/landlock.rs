use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct LandlockSupport {
    pub supported: bool,
    pub abi: Option<u32>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LandlockFilesystemPolicy {
    pub exec_paths: Vec<String>,
    pub read_paths: Vec<String>,
    pub write_paths: Vec<String>,
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
    apply_filesystem_policy(&LandlockFilesystemPolicy {
        exec_paths: Vec::new(),
        read_paths: read_paths.to_vec(),
        write_paths: write_paths.to_vec(),
    })
}

/// Apply Landlock restrictions that only handle write-related access (reads remain unrestricted by
/// Landlock). This is useful for "workspace" isolation where host paths are visible but should not
/// be writable.
pub fn apply_write_only_allowlist(write_paths: &[String]) -> LandlockApplyReport {
    #[cfg(target_os = "linux")]
    {
        linux::apply_write_only_allowlist(write_paths)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = write_paths;
        LandlockApplyReport {
            support: detect_support(),
            attempted: false,
            applied: false,
            rules_added: 0,
            reason: Some("landlock only supported on Linux".to_string()),
        }
    }
}

pub fn apply_filesystem_policy(policy: &LandlockFilesystemPolicy) -> LandlockApplyReport {
    #[cfg(target_os = "linux")]
    {
        linux::apply_filesystem_policy(policy)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = policy;
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
    use super::{LandlockApplyReport, LandlockFilesystemPolicy, LandlockSupport};
    use linux_raw_sys::{general, landlock};
    use std::collections::BTreeMap;
    use std::ffi::CString;
    use std::mem;
    use std::os::fd::RawFd;

    fn abi_supported_access_fs(abi: u32) -> u64 {
        let mut mask = landlock::LANDLOCK_ACCESS_FS_EXECUTE as u64
            | landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64
            | landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
            | landlock::LANDLOCK_ACCESS_FS_READ_DIR as u64
            | landlock::LANDLOCK_ACCESS_FS_REMOVE_DIR as u64
            | landlock::LANDLOCK_ACCESS_FS_REMOVE_FILE as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_CHAR as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_DIR as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_REG as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_SOCK as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_FIFO as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_BLOCK as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_SYM as u64;

        if abi >= 2 {
            mask |= landlock::LANDLOCK_ACCESS_FS_REFER as u64;
        }
        if abi >= 3 {
            mask |= landlock::LANDLOCK_ACCESS_FS_TRUNCATE as u64;
        }

        mask
    }

    fn exec_access_mask() -> u64 {
        landlock::LANDLOCK_ACCESS_FS_EXECUTE as u64
    }

    fn read_access_mask(abi: u32) -> u64 {
        let supported = abi_supported_access_fs(abi);
        supported
            & (landlock::LANDLOCK_ACCESS_FS_EXECUTE as u64
                | landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
                | landlock::LANDLOCK_ACCESS_FS_READ_DIR as u64)
    }

    fn write_access_mask(abi: u32) -> u64 {
        let supported = abi_supported_access_fs(abi);

        let mut mask = landlock::LANDLOCK_ACCESS_FS_EXECUTE as u64
            | landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
            | landlock::LANDLOCK_ACCESS_FS_READ_DIR as u64
            | landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64
            | landlock::LANDLOCK_ACCESS_FS_REMOVE_DIR as u64
            | landlock::LANDLOCK_ACCESS_FS_REMOVE_FILE as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_CHAR as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_DIR as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_REG as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_SOCK as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_FIFO as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_BLOCK as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_SYM as u64;

        if abi >= 2 {
            mask |= landlock::LANDLOCK_ACCESS_FS_REFER as u64;
        }
        if abi >= 3 {
            mask |= landlock::LANDLOCK_ACCESS_FS_TRUNCATE as u64;
        }

        supported & mask
    }

    fn write_only_access_mask(abi: u32) -> u64 {
        let supported = abi_supported_access_fs(abi);

        let mut mask = landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64
            | landlock::LANDLOCK_ACCESS_FS_REMOVE_DIR as u64
            | landlock::LANDLOCK_ACCESS_FS_REMOVE_FILE as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_CHAR as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_DIR as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_REG as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_SOCK as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_FIFO as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_BLOCK as u64
            | landlock::LANDLOCK_ACCESS_FS_MAKE_SYM as u64;

        if abi >= 2 {
            mask |= landlock::LANDLOCK_ACCESS_FS_REFER as u64;
        }
        if abi >= 3 {
            mask |= landlock::LANDLOCK_ACCESS_FS_TRUNCATE as u64;
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

    pub(super) fn apply_filesystem_policy(
        policy: &LandlockFilesystemPolicy,
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

        if policy.exec_paths.is_empty()
            && policy.read_paths.is_empty()
            && policy.write_paths.is_empty()
        {
            return LandlockApplyReport {
                support,
                attempted: false,
                applied: false,
                rules_added: 0,
                reason: Some("landlock policy was empty; skipping".to_string()),
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

        let exec_mask = exec_access_mask();
        let read_mask = read_access_mask(abi);
        let write_mask = write_access_mask(abi);

        let mut allowlist: BTreeMap<&str, u64> = BTreeMap::new();
        for path in &policy.exec_paths {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                continue;
            }
            *allowlist.entry(trimmed).or_default() |= exec_mask;
        }
        for path in &policy.read_paths {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                continue;
            }
            *allowlist.entry(trimmed).or_default() |= read_mask;
        }
        for path in &policy.write_paths {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                continue;
            }
            *allowlist.entry(trimmed).or_default() |= write_mask;
        }

        let handled_access_fs =
            allowlist.values().fold(0u64, |acc, mask| acc | *mask) & abi_supported_access_fs(abi);

        let ruleset_attr = landlock::landlock_ruleset_attr { handled_access_fs };
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

            let attr = landlock::landlock_path_beneath_attr {
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

    pub(super) fn apply_write_only_allowlist(write_paths: &[String]) -> LandlockApplyReport {
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

        if write_paths.iter().all(|p| p.trim().is_empty()) {
            return LandlockApplyReport {
                support,
                attempted: false,
                applied: false,
                rules_added: 0,
                reason: Some("landlock policy was empty; skipping".to_string()),
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

        let write_mask = write_only_access_mask(abi);
        let mut allowlist: BTreeMap<&str, u64> = BTreeMap::new();
        for path in write_paths {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                continue;
            }
            *allowlist.entry(trimmed).or_default() |= write_mask;
        }

        let handled_access_fs =
            allowlist.values().fold(0u64, |acc, mask| acc | *mask) & abi_supported_access_fs(abi);

        let ruleset_attr = landlock::landlock_ruleset_attr { handled_access_fs };
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

            let attr = landlock::landlock_path_beneath_attr {
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
            general::__NR_landlock_create_ruleset as libc::c_long,
            std::ptr::null::<libc::c_void>(),
            0usize,
            landlock::LANDLOCK_CREATE_RULESET_VERSION,
        );
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(format!("landlock unavailable: {err}"));
        }
        Ok(ret as u32)
    }

    unsafe fn landlock_create_ruleset(
        attr: &landlock::landlock_ruleset_attr,
    ) -> Result<RawFd, String> {
        let ret = libc::syscall(
            general::__NR_landlock_create_ruleset as libc::c_long,
            attr as *const landlock::landlock_ruleset_attr,
            mem::size_of::<landlock::landlock_ruleset_attr>(),
            0u32,
        );
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(format!("landlock create_ruleset failed: {err}"));
        }
        Ok(ret as RawFd)
    }

    unsafe fn landlock_add_rule(
        ruleset_fd: RawFd,
        attr: &landlock::landlock_path_beneath_attr,
    ) -> Result<(), String> {
        let ret = libc::syscall(
            general::__NR_landlock_add_rule as libc::c_long,
            ruleset_fd,
            landlock::landlock_rule_type::LANDLOCK_RULE_PATH_BENEATH as u32,
            attr as *const landlock::landlock_path_beneath_attr,
            0u32,
        );
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(format!("landlock add_rule failed: {err}"));
        }
        Ok(())
    }

    unsafe fn landlock_restrict_self(ruleset_fd: RawFd) -> Result<(), String> {
        let ret = libc::syscall(
            general::__NR_landlock_restrict_self as libc::c_long,
            ruleset_fd,
            0u32,
        );
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

    fn open_opath(path: &str) -> Result<RawFd, OpenError> {
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
