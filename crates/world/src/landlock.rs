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
    pub discover_paths: Vec<String>,
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
        discover_paths: read_paths.to_vec(),
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

    fn discover_access_mask(abi: u32) -> u64 {
        let supported = abi_supported_access_fs(abi);
        supported & (landlock::LANDLOCK_ACCESS_FS_READ_DIR as u64)
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

    fn regular_file_access_mask(abi: u32) -> u64 {
        abi_supported_access_fs(abi)
            & (landlock::LANDLOCK_ACCESS_FS_EXECUTE as u64
                | landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64
                | landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
                | landlock::LANDLOCK_ACCESS_FS_TRUNCATE as u64)
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum PathRuleTargetKind {
        RegularFile,
        Directory,
    }

    fn classify_path_rule_target(fd: RawFd, path: &str) -> Result<PathRuleTargetKind, String> {
        let mut stat = unsafe { mem::zeroed::<libc::stat>() };
        if unsafe { libc::fstat(fd, &mut stat) } < 0 {
            return Err(format!(
                "failed to inspect {path:?} for landlock: {}",
                std::io::Error::last_os_error()
            ));
        }

        match stat.st_mode & libc::S_IFMT {
            libc::S_IFREG => Ok(PathRuleTargetKind::RegularFile),
            libc::S_IFDIR => Ok(PathRuleTargetKind::Directory),
            libc::S_IFLNK => Err(format!(
                "refusing symlink {path:?} as a landlock rule target"
            )),
            _ => Err(format!(
                "unsupported file type for landlock rule target {path:?}"
            )),
        }
    }

    fn compatible_path_rule_access(
        kind: PathRuleTargetKind,
        requested_access: u64,
        abi: u32,
    ) -> Result<u64, String> {
        let abi_supported_request = requested_access & abi_supported_access_fs(abi);
        if requested_access != 0 && abi_supported_request == 0 {
            return Err("landlock rule has no ABI-supported access".to_string());
        }
        let compatible_access = match kind {
            PathRuleTargetKind::RegularFile => {
                abi_supported_request & regular_file_access_mask(abi)
            }
            PathRuleTargetKind::Directory => abi_supported_request,
        };
        if abi_supported_request != 0 && compatible_access == 0 {
            return Err(match kind {
                PathRuleTargetKind::RegularFile => {
                    "regular-file landlock rule requested only directory-compatible access"
                        .to_string()
                }
                PathRuleTargetKind::Directory => {
                    "directory landlock rule has no ABI-supported access".to_string()
                }
            });
        }
        Ok(compatible_access)
    }

    fn open_path_rule(
        path: &str,
        requested_access: u64,
        abi: u32,
    ) -> Result<Option<(RawFd, u64)>, String> {
        let fd = match open_opath(path) {
            Ok(fd) => fd,
            Err(OpenError::NotFound) => return Ok(None),
            Err(OpenError::Other(error)) => return Err(error),
        };
        let access = classify_path_rule_target(fd, path)
            .and_then(|kind| compatible_path_rule_access(kind, requested_access, abi));
        match access {
            Ok(access) => Ok(Some((fd, access))),
            Err(error) => {
                unsafe {
                    libc::close(fd);
                }
                Err(error)
            }
        }
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
            && policy.discover_paths.is_empty()
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
        let discover_mask = discover_access_mask(abi);
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
        for path in &policy.discover_paths {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                continue;
            }
            *allowlist.entry(trimmed).or_default() |= discover_mask;
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

            let (fd, access) = match open_path_rule(path, access, abi) {
                Ok(Some(rule)) => rule,
                Ok(None) => continue,
                Err(err) => {
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
                    reason: Some(format!("landlock rule for {path:?} failed: {err}")),
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

            let (fd, access) = match open_path_rule(path, access, abi) {
                Ok(Some(rule)) => rule,
                Ok(None) => continue,
                Err(err) => {
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
                    reason: Some(format!("landlock rule for {path:?} failed: {err}")),
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
        let how = general::open_how {
            flags: (libc::O_PATH | libc::O_CLOEXEC | libc::O_NOFOLLOW) as u64,
            mode: 0,
            resolve: general::RESOLVE_NO_SYMLINKS as u64,
        };
        let fd = unsafe {
            libc::syscall(
                general::__NR_openat2 as libc::c_long,
                libc::AT_FDCWD,
                cstr.as_ptr(),
                &how as *const general::open_how,
                mem::size_of::<general::open_how>(),
            )
        };
        if fd < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::NotFound {
                return Err(OpenError::NotFound);
            }
            return Err(OpenError::Other(format!(
                "failed to open {path:?} without following symlinks for landlock: {err}"
            )));
        }
        Ok(fd as RawFd)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs;
        use std::os::unix::fs::symlink;

        #[test]
        fn exact_regular_file_masks_are_file_compatible() {
            let abi = 7;
            let read = compatible_path_rule_access(
                PathRuleTargetKind::RegularFile,
                read_access_mask(abi),
                abi,
            )
            .unwrap();
            assert_eq!(
                read,
                landlock::LANDLOCK_ACCESS_FS_EXECUTE as u64
                    | landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
            );
            assert_eq!(read & landlock::LANDLOCK_ACCESS_FS_READ_DIR as u64, 0);

            let write = compatible_path_rule_access(
                PathRuleTargetKind::RegularFile,
                write_access_mask(abi),
                abi,
            )
            .unwrap();
            for right in [
                landlock::LANDLOCK_ACCESS_FS_EXECUTE,
                landlock::LANDLOCK_ACCESS_FS_READ_FILE,
                landlock::LANDLOCK_ACCESS_FS_WRITE_FILE,
                landlock::LANDLOCK_ACCESS_FS_TRUNCATE,
            ] {
                assert_ne!(write & right as u64, 0, "missing file right {right}");
            }
            for right in [
                landlock::LANDLOCK_ACCESS_FS_READ_DIR,
                landlock::LANDLOCK_ACCESS_FS_REMOVE_FILE,
                landlock::LANDLOCK_ACCESS_FS_MAKE_REG,
                landlock::LANDLOCK_ACCESS_FS_REFER,
            ] {
                assert_eq!(write & right as u64, 0, "retained directory right {right}");
            }

            assert_eq!(
                compatible_path_rule_access(
                    PathRuleTargetKind::RegularFile,
                    exec_access_mask(),
                    abi,
                )
                .unwrap(),
                landlock::LANDLOCK_ACCESS_FS_EXECUTE as u64
            );
        }

        #[test]
        fn directory_masks_are_unchanged_and_abi_bounded() {
            for abi in [1, 2, 3, 7] {
                let requested = write_access_mask(abi);
                assert_eq!(
                    compatible_path_rule_access(PathRuleTargetKind::Directory, requested, abi)
                        .unwrap(),
                    requested
                );
                assert_eq!(requested & !abi_supported_access_fs(abi), 0);
            }
            assert_eq!(
                write_access_mask(1) & landlock::LANDLOCK_ACCESS_FS_TRUNCATE as u64,
                0
            );
            assert_ne!(
                write_access_mask(3) & landlock::LANDLOCK_ACCESS_FS_TRUNCATE as u64,
                0
            );
        }

        #[test]
        fn incompatible_or_unsupported_masks_fail_closed() {
            assert!(compatible_path_rule_access(
                PathRuleTargetKind::RegularFile,
                landlock::LANDLOCK_ACCESS_FS_READ_DIR as u64,
                7,
            )
            .is_err());
            assert!(
                compatible_path_rule_access(PathRuleTargetKind::Directory, 1u64 << 63, 7,).is_err()
            );
        }

        #[test]
        fn target_classification_rejects_symlinks_and_unsupported_types() {
            let root = tempfile::tempdir().unwrap();
            let file = root.path().join("file");
            let directory = root.path().join("directory");
            let link = root.path().join("link");
            let fifo = root.path().join("fifo");
            fs::write(&file, "data").unwrap();
            fs::create_dir(&directory).unwrap();
            symlink(&file, &link).unwrap();
            let fifo_cstr = CString::new(fifo.as_os_str().as_encoded_bytes()).unwrap();
            assert_eq!(unsafe { libc::mkfifo(fifo_cstr.as_ptr(), 0o600) }, 0);

            for (path, expected) in [
                (&file, Some(PathRuleTargetKind::RegularFile)),
                (&directory, Some(PathRuleTargetKind::Directory)),
                (&link, None),
                (&fifo, None),
            ] {
                let path = path.display().to_string();
                let fd = match open_opath(&path) {
                    Ok(fd) => fd,
                    Err(_) => panic!("open test target {path}"),
                };
                let classified = classify_path_rule_target(fd, &path);
                unsafe {
                    libc::close(fd);
                }
                match expected {
                    Some(expected) => assert_eq!(classified.unwrap(), expected),
                    None => assert!(classified.is_err()),
                }
            }
        }

        #[test]
        fn trusted_open_rejects_intermediate_symlink() {
            let root = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            fs::write(outside.path().join("secret"), "secret").unwrap();
            symlink(outside.path(), root.path().join("escape")).unwrap();

            let escaped = root.path().join("escape/secret");
            assert!(open_opath(&escaped.display().to_string()).is_err());
        }
    }
}
