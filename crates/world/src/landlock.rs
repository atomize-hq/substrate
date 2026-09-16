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
        NullDevice,
        UrandomDevice,
    }

    impl PathRuleTargetKind {
        fn fixed_support_device(path: &str) -> Option<Self> {
            match path {
                "/dev/null" => Some(Self::NullDevice),
                "/dev/urandom" => Some(Self::UrandomDevice),
                _ => None,
            }
        }

        fn fixed_support_device_component(self) -> Option<&'static str> {
            match self {
                Self::NullDevice => Some("null"),
                Self::UrandomDevice => Some("urandom"),
                Self::RegularFile | Self::Directory => None,
            }
        }
    }

    fn fixed_support_device_selection(
        policy: &LandlockFilesystemPolicy,
    ) -> Result<BTreeMap<&'static str, u64>, String> {
        let selected =
            |path: &str, paths: &[String]| paths.iter().any(|candidate| candidate.trim() == path);
        let mut devices = BTreeMap::new();
        for (path, kind) in [
            ("/dev/null", PathRuleTargetKind::NullDevice),
            ("/dev/urandom", PathRuleTargetKind::UrandomDevice),
        ] {
            let exec = selected(path, &policy.exec_paths);
            let discover = selected(path, &policy.discover_paths);
            let read = selected(path, &policy.read_paths);
            let write = selected(path, &policy.write_paths);
            if !exec && !discover && !read && !write {
                continue;
            }
            if exec {
                return Err(format!("fixed support device {path:?} cannot be executed"));
            }
            if kind == PathRuleTargetKind::UrandomDevice && write {
                return Err("fixed support device \"/dev/urandom\" cannot be written".to_string());
            }
            let mut access = 0;
            if read {
                access |= landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64;
            }
            if write {
                access |= landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64;
            }
            if access == 0 {
                let reason = if discover {
                    "discovery-only selection"
                } else {
                    "unsupported selection"
                };
                return Err(format!("fixed support device {path:?} has {reason}"));
            }
            devices.insert(path, access);
        }
        Ok(devices)
    }

    fn classify_path_rule_target(fd: RawFd, path: &str) -> Result<PathRuleTargetKind, String> {
        let mut stat = unsafe { mem::zeroed::<libc::stat>() };
        if unsafe { libc::fstat(fd, &mut stat) } < 0 {
            return Err(format!(
                "failed to inspect {path:?} for landlock: {}",
                std::io::Error::last_os_error()
            ));
        }

        if let Some(kind) = PathRuleTargetKind::fixed_support_device(path) {
            let (major, minor) = match kind {
                PathRuleTargetKind::NullDevice => (1, 3),
                PathRuleTargetKind::UrandomDevice => (1, 9),
                PathRuleTargetKind::RegularFile | PathRuleTargetKind::Directory => unreachable!(),
            };
            if stat.st_mode & libc::S_IFMT != libc::S_IFCHR
                || libc::major(stat.st_rdev) != major
                || libc::minor(stat.st_rdev) != minor
            {
                return Err(format!(
                    "fixed support device identity mismatch for landlock rule target {path:?}"
                ));
            }
            return Ok(kind);
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
            PathRuleTargetKind::NullDevice => {
                abi_supported_request
                    & (landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
                        | landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64)
            }
            PathRuleTargetKind::UrandomDevice => {
                abi_supported_request & landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
            }
        };
        if (abi_supported_request != 0
            || matches!(
                kind,
                PathRuleTargetKind::NullDevice | PathRuleTargetKind::UrandomDevice
            ))
            && compatible_access == 0
        {
            return Err(match kind {
                PathRuleTargetKind::RegularFile => {
                    "regular-file landlock rule requested only directory-compatible access"
                        .to_string()
                }
                PathRuleTargetKind::Directory => {
                    "directory landlock rule has no ABI-supported access".to_string()
                }
                PathRuleTargetKind::NullDevice | PathRuleTargetKind::UrandomDevice => {
                    "fixed support device landlock rule requested no compatible access".to_string()
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
        let fd = match PathRuleTargetKind::fixed_support_device(path) {
            Some(kind) => open_fixed_support_device_rule(kind)?,
            None => match open_opath(path) {
                Ok(fd) => fd,
                Err(OpenError::NotFound) => return Ok(None),
                Err(OpenError::Other(error)) => return Err(error),
            },
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

        let fixed_devices = match fixed_support_device_selection(policy) {
            Ok(devices) => devices,
            Err(reason) => {
                return LandlockApplyReport {
                    support,
                    attempted: true,
                    applied: false,
                    rules_added: 0,
                    reason: Some(reason),
                };
            }
        };
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

        let mut handled_access_fs =
            allowlist.values().fold(0u64, |acc, mask| acc | *mask) & abi_supported_access_fs(abi);
        if !fixed_devices.is_empty() {
            handled_access_fs |= landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
                | landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64;
        }

        for (path, access) in &fixed_devices {
            allowlist.insert(path, *access);
        }

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
        let policy = LandlockFilesystemPolicy {
            exec_paths: Vec::new(),
            discover_paths: Vec::new(),
            read_paths: Vec::new(),
            write_paths: write_paths.to_vec(),
        };
        let fixed_devices = match fixed_support_device_selection(&policy) {
            Ok(devices) => devices,
            Err(reason) => {
                return LandlockApplyReport {
                    support,
                    attempted: true,
                    applied: false,
                    rules_added: 0,
                    reason: Some(reason),
                };
            }
        };
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

        for (path, access) in fixed_devices {
            allowlist.insert(path, access);
        }

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

    fn open_fixed_support_device_rule(kind: PathRuleTargetKind) -> Result<RawFd, String> {
        open_fixed_support_device_rule_at("/dev", kind)
    }

    fn open_fixed_support_device_rule_at(
        root: &str,
        kind: PathRuleTargetKind,
    ) -> Result<RawFd, String> {
        let component = kind
            .fixed_support_device_component()
            .ok_or_else(|| "not a fixed support device rule target".to_string())?;
        let root = CString::new(root)
            .map_err(|error| format!("invalid fixed support device root: {error}"))?;
        let root_fd = unsafe {
            libc::open(
                root.as_ptr(),
                libc::O_PATH | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if root_fd < 0 {
            return Err(format!(
                "failed to open fixed support device root without following symlinks: {}",
                std::io::Error::last_os_error()
            ));
        }
        let component = CString::new(component).expect("fixed device component has no NUL");
        let how = general::open_how {
            flags: (libc::O_PATH | libc::O_CLOEXEC | libc::O_NOFOLLOW) as u64,
            mode: 0,
            resolve: (general::RESOLVE_BENEATH
                | general::RESOLVE_NO_MAGICLINKS
                | general::RESOLVE_NO_SYMLINKS
                | general::RESOLVE_NO_XDEV) as u64,
        };
        let fd = unsafe {
            libc::syscall(
                general::__NR_openat2 as libc::c_long,
                root_fd,
                component.as_ptr(),
                &how as *const general::open_how,
                mem::size_of::<general::open_how>(),
            )
        };
        unsafe {
            libc::close(root_fd);
        }
        if fd < 0 {
            return Err(format!(
                "failed to resolve fixed support device {component:?}: {}",
                std::io::Error::last_os_error()
            ));
        }
        let fd = fd as RawFd;
        let mut stat = unsafe { mem::zeroed::<libc::stat>() };
        if unsafe { libc::fstat(fd, &mut stat) } < 0 {
            unsafe {
                libc::close(fd);
            }
            return Err(format!(
                "failed to inspect resolved fixed support device {component:?}: {}",
                std::io::Error::last_os_error()
            ));
        }
        if stat.st_mode & libc::S_IFMT == libc::S_IFLNK {
            unsafe {
                libc::close(fd);
            }
            return Err(format!(
                "refusing symlink fixed support device {component:?}"
            ));
        }
        Ok(fd)
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
        use std::os::fd::AsRawFd;
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

        #[test]
        fn fixed_support_device_selection_keeps_explicit_rights() {
            let policy = LandlockFilesystemPolicy {
                exec_paths: Vec::new(),
                discover_paths: vec!["/dev/null".to_string(), "/dev/urandom".to_string()],
                read_paths: vec!["/dev/null".to_string(), "/dev/urandom".to_string()],
                write_paths: vec!["/dev/null".to_string()],
            };
            let selected = fixed_support_device_selection(&policy).unwrap();
            assert_eq!(
                selected["/dev/null"],
                landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
                    | landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64
            );
            assert_eq!(
                selected["/dev/urandom"],
                landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
            );
            for abi in [1, 2, 3, 7] {
                assert_eq!(
                    compatible_path_rule_access(
                        PathRuleTargetKind::NullDevice,
                        landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
                            | landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64,
                        abi,
                    )
                    .unwrap(),
                    landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
                        | landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64
                );
                assert_eq!(
                    compatible_path_rule_access(
                        PathRuleTargetKind::UrandomDevice,
                        landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64,
                        abi,
                    )
                    .unwrap(),
                    landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
                );
            }
            assert!(selected.values().all(|access| {
                access
                    & !(landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
                        | landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64)
                    == 0
            }));
            assert_eq!(
                compatible_path_rule_access(
                    PathRuleTargetKind::NullDevice,
                    read_access_mask(7),
                    7,
                )
                .unwrap(),
                landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
            );
            assert_eq!(
                compatible_path_rule_access(
                    PathRuleTargetKind::UrandomDevice,
                    write_access_mask(7),
                    7,
                )
                .unwrap(),
                landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64
            );

            for policy in [
                LandlockFilesystemPolicy {
                    exec_paths: vec!["/dev/null".to_string()],
                    discover_paths: Vec::new(),
                    read_paths: Vec::new(),
                    write_paths: Vec::new(),
                },
                LandlockFilesystemPolicy {
                    exec_paths: Vec::new(),
                    discover_paths: Vec::new(),
                    read_paths: Vec::new(),
                    write_paths: vec!["/dev/urandom".to_string()],
                },
                LandlockFilesystemPolicy {
                    exec_paths: Vec::new(),
                    discover_paths: vec!["/dev/null".to_string()],
                    read_paths: Vec::new(),
                    write_paths: Vec::new(),
                },
                LandlockFilesystemPolicy {
                    exec_paths: vec!["/dev/null".to_string()],
                    discover_paths: Vec::new(),
                    read_paths: vec!["/dev/null".to_string()],
                    write_paths: Vec::new(),
                },
                LandlockFilesystemPolicy {
                    exec_paths: Vec::new(),
                    discover_paths: vec!["/dev/urandom".to_string()],
                    read_paths: Vec::new(),
                    write_paths: Vec::new(),
                },
                LandlockFilesystemPolicy {
                    exec_paths: Vec::new(),
                    discover_paths: Vec::new(),
                    read_paths: vec!["/dev/urandom".to_string()],
                    write_paths: vec!["/dev/urandom".to_string()],
                },
            ] {
                assert!(fixed_support_device_selection(&policy).is_err());
            }
            assert!(compatible_path_rule_access(
                PathRuleTargetKind::NullDevice,
                landlock::LANDLOCK_ACCESS_FS_READ_DIR as u64,
                7,
            )
            .is_err());
            assert!(compatible_path_rule_access(
                PathRuleTargetKind::UrandomDevice,
                landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64,
                7,
            )
            .is_err());
            assert!(
                compatible_path_rule_access(PathRuleTargetKind::NullDevice, 1u64 << 63, 7,)
                    .is_err()
            );
            assert!(compatible_path_rule_access(PathRuleTargetKind::NullDevice, 0, 7).is_err());
        }

        #[test]
        fn fixed_support_devices_require_the_exact_rule_descriptor_identity() {
            for (path, expected) in [
                ("/dev/null", PathRuleTargetKind::NullDevice),
                ("/dev/urandom", PathRuleTargetKind::UrandomDevice),
            ] {
                let fd = open_fixed_support_device_rule(expected).unwrap();
                assert_eq!(classify_path_rule_target(fd, path).unwrap(), expected);
                unsafe { libc::close(fd) };
            }

            let null = open_fixed_support_device_rule(PathRuleTargetKind::NullDevice).unwrap();
            assert!(classify_path_rule_target(null, "/dev/urandom").is_err());
            unsafe { libc::close(null) };

            let urandom =
                open_fixed_support_device_rule(PathRuleTargetKind::UrandomDevice).unwrap();
            assert!(classify_path_rule_target(urandom, "/dev/null").is_err());
            unsafe { libc::close(urandom) };

            let zero = match open_opath("/dev/zero") {
                Ok(fd) => fd,
                Err(_) => panic!("open /dev/zero"),
            };
            assert!(classify_path_rule_target(zero, "/dev/null").is_err());
            assert!(classify_path_rule_target(zero, "/dev/zero").is_err());
            unsafe { libc::close(zero) };

            let (fd, access) = open_path_rule("/dev/null", read_access_mask(7), 7)
                .unwrap()
                .unwrap();
            assert_eq!(access, landlock::LANDLOCK_ACCESS_FS_READ_FILE as u64);
            assert_eq!(
                classify_path_rule_target(fd, "/dev/null").unwrap(),
                PathRuleTargetKind::NullDevice
            );
            unsafe { libc::close(fd) };
            assert!(open_path_rule(
                "/dev/urandom",
                landlock::LANDLOCK_ACCESS_FS_WRITE_FILE as u64,
                7,
            )
            .is_err());
        }

        #[test]
        fn fixed_support_device_resolver_rejects_substitution_and_links() {
            let root = tempfile::tempdir().unwrap();
            fs::write(root.path().join("null"), "not a device").unwrap();
            fs::create_dir(root.path().join("urandom")).unwrap();
            let fd = open_fixed_support_device_rule_at(
                &root.path().display().to_string(),
                PathRuleTargetKind::NullDevice,
            )
            .unwrap();
            assert!(classify_path_rule_target(fd, "/dev/null").is_err());
            unsafe { libc::close(fd) };
            let fd = open_fixed_support_device_rule_at(
                &root.path().display().to_string(),
                PathRuleTargetKind::UrandomDevice,
            )
            .unwrap();
            assert!(classify_path_rule_target(fd, "/dev/urandom").is_err());
            unsafe { libc::close(fd) };
            fs::remove_file(root.path().join("null")).unwrap();
            symlink("/dev/null", root.path().join("null")).unwrap();
            assert!(open_fixed_support_device_rule_at(
                &root.path().display().to_string(),
                PathRuleTargetKind::NullDevice,
            )
            .is_err());
            fs::remove_dir(root.path().join("urandom")).unwrap();
            assert!(open_fixed_support_device_rule_at(
                &root.path().display().to_string(),
                PathRuleTargetKind::UrandomDevice,
            )
            .is_err());
            let linked_root = root.path().with_extension("link");
            symlink(root.path(), &linked_root).unwrap();
            assert!(open_fixed_support_device_rule_at(
                &linked_root.display().to_string(),
                PathRuleTargetKind::NullDevice,
            )
            .is_err());
            let held_root = fs::File::open(root.path()).unwrap();
            let magic_root = format!("/proc/self/fd/{}", held_root.as_raw_fd());
            assert!(
                open_fixed_support_device_rule_at(&magic_root, PathRuleTargetKind::NullDevice,)
                    .is_err()
            );
        }
    }
}
