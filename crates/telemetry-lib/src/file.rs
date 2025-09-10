use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::time::Instant;

use crate::{c_str_to_string, log_syscall};

// Type definitions for original functions
type OpenFn = unsafe extern "C" fn(*const c_char, c_int, libc::mode_t) -> c_int;
type OpenFn2 = unsafe extern "C" fn(*const c_char, c_int) -> c_int;
type OpenatFn = unsafe extern "C" fn(c_int, *const c_char, c_int, libc::mode_t) -> c_int;
type OpenatFn2 = unsafe extern "C" fn(c_int, *const c_char, c_int) -> c_int;
type CreatFn = unsafe extern "C" fn(*const c_char, libc::mode_t) -> c_int;
type UnlinkFn = unsafe extern "C" fn(*const c_char) -> c_int;
type RenameFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;

// Get original function pointer via dlsym
unsafe fn get_original<T>(name: &str) -> Option<T> {
    let c_name = CString::new(name).ok()?;
    let ptr = libc::dlsym(libc::RTLD_NEXT, c_name.as_ptr());
    if ptr.is_null() {
        None
    } else {
        Some(std::mem::transmute_copy(&ptr))
    }
}

// Helper to convert flags to readable string
fn flags_to_string(flags: c_int) -> String {
    let mut parts = Vec::new();

    // Check access mode (O_RDONLY = 0, O_WRONLY = 1, O_RDWR = 2)
    let access_mode = flags & libc::O_ACCMODE;
    match access_mode {
        libc::O_RDONLY => parts.push("O_RDONLY"),
        libc::O_WRONLY => parts.push("O_WRONLY"),
        libc::O_RDWR => parts.push("O_RDWR"),
        _ => {}
    }
    if flags & libc::O_CREAT == libc::O_CREAT {
        parts.push("O_CREAT");
    }
    if flags & libc::O_TRUNC == libc::O_TRUNC {
        parts.push("O_TRUNC");
    }
    if flags & libc::O_APPEND == libc::O_APPEND {
        parts.push("O_APPEND");
    }

    if parts.is_empty() {
        format!("0x{:x}", flags)
    } else {
        parts.join("|")
    }
}

/// Intercept libc `open` to log filesystem access and skip tracing our own files.
///
/// # Safety
/// - `path` must be a valid, non-null C string pointer.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn open(path: *const c_char, flags: c_int, mode: c_int) -> c_int {
    let start = Instant::now();
    let path_str = c_str_to_string(path);
    let flags_str = flags_to_string(flags);

    // Check if we should skip logging for certain paths
    // Skip our own trace file and system files
    if path_str.contains(".substrate/trace.jsonl")
        || path_str.starts_with("/proc/")
        || path_str.starts_with("/sys/")
        || path_str.starts_with("/dev/")
    {
        // Call original without logging
        let ptr = unsafe {
            let c_name = CString::new("open").unwrap();
            libc::dlsym(libc::RTLD_NEXT, c_name.as_ptr())
        };
        if !ptr.is_null() {
            if flags & libc::O_CREAT != 0 {
                let original_fn: OpenFn = unsafe { std::mem::transmute(ptr) };
                return unsafe { original_fn(path, flags, mode as libc::mode_t) };
            } else {
                let original_fn: OpenFn2 = unsafe { std::mem::transmute(ptr) };
                return unsafe { original_fn(path, flags) };
            }
        }
        return -1;
    }

    // Log the attempt
    log_syscall(
        "open",
        vec![path_str.clone(), flags_str.clone()],
        None,
        None,
        start.elapsed().as_micros() as u64,
    );

    // Call original open
    let ptr = unsafe {
        let c_name = CString::new("open").unwrap();
        libc::dlsym(libc::RTLD_NEXT, c_name.as_ptr())
    };
    let result = if !ptr.is_null() {
        if flags & libc::O_CREAT != 0 {
            let original_fn: OpenFn = unsafe { std::mem::transmute(ptr) };
            unsafe { original_fn(path, flags, mode as libc::mode_t) }
        } else {
            let original_fn: OpenFn2 = unsafe { std::mem::transmute(ptr) };
            unsafe { original_fn(path, flags) }
        }
    } else {
        -1
    };

    let elapsed = start.elapsed().as_micros() as u64;
    if result >= 0 {
        log_syscall(
            "open",
            vec![path_str, flags_str],
            Some(format!("fd: {}", result)),
            None,
            elapsed,
        );
    } else {
        let error = std::io::Error::last_os_error().to_string();
        log_syscall(
            "open",
            vec![path_str, flags_str],
            Some(format!("failed: {}", result)),
            Some(error),
            elapsed,
        );
    }

    result
}

/// Intercept libc `openat` to log filesystem access and skip tracing our own files.
///
/// # Safety
/// - `path` must be a valid, non-null C string pointer.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn openat(
    dirfd: c_int,
    path: *const c_char,
    flags: c_int,
    mode: c_int,
) -> c_int {
    let start = Instant::now();
    let path_str = c_str_to_string(path);
    let flags_str = flags_to_string(flags);
    let dirfd_str = if dirfd == libc::AT_FDCWD {
        "AT_FDCWD".to_string()
    } else {
        format!("{}", dirfd)
    };

    // Skip system paths
    if path_str.contains(".substrate/trace.jsonl")
        || path_str.starts_with("/proc/")
        || path_str.starts_with("/sys/")
        || path_str.starts_with("/dev/")
    {
        let ptr = unsafe {
            let c_name = CString::new("openat").unwrap();
            libc::dlsym(libc::RTLD_NEXT, c_name.as_ptr())
        };
        if !ptr.is_null() {
            if flags & libc::O_CREAT != 0 {
                let original_fn: OpenatFn = unsafe { std::mem::transmute(ptr) };
                return unsafe { original_fn(dirfd, path, flags, mode as libc::mode_t) };
            } else {
                let original_fn: OpenatFn2 = unsafe { std::mem::transmute(ptr) };
                return unsafe { original_fn(dirfd, path, flags) };
            }
        }
        return -1;
    }

    // Log the attempt
    log_syscall(
        "openat",
        vec![dirfd_str.clone(), path_str.clone(), flags_str.clone()],
        None,
        None,
        start.elapsed().as_micros() as u64,
    );

    // Call original openat
    let ptr = unsafe {
        let c_name = CString::new("openat").unwrap();
        libc::dlsym(libc::RTLD_NEXT, c_name.as_ptr())
    };
    let result = if !ptr.is_null() {
        if flags & libc::O_CREAT != 0 {
            let original_fn: OpenatFn = unsafe { std::mem::transmute(ptr) };
            unsafe { original_fn(dirfd, path, flags, mode as libc::mode_t) }
        } else {
            let original_fn: OpenatFn2 = unsafe { std::mem::transmute(ptr) };
            unsafe { original_fn(dirfd, path, flags) }
        }
    } else {
        -1
    };

    let elapsed = start.elapsed().as_micros() as u64;
    if result >= 0 {
        log_syscall(
            "openat",
            vec![dirfd_str, path_str, flags_str],
            Some(format!("fd: {}", result)),
            None,
            elapsed,
        );
    } else {
        let error = std::io::Error::last_os_error().to_string();
        log_syscall(
            "openat",
            vec![dirfd_str, path_str, flags_str],
            Some(format!("failed: {}", result)),
            Some(error),
            elapsed,
        );
    }

    result
}

/// Intercept libc `creat` to log file creation.
///
/// # Safety
/// - `path` must be a valid, non-null C string pointer.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn creat(path: *const c_char, mode: libc::mode_t) -> c_int {
    let start = Instant::now();
    let path_str = c_str_to_string(path);

    // Skip system paths
    if path_str.contains(".substrate/") {
        if let Some(original_fn) = get_original::<CreatFn>("creat") {
            return original_fn(path, mode);
        }
        return -1;
    }

    // Log the attempt
    log_syscall(
        "creat",
        vec![path_str.clone(), format!("{:o}", mode)],
        None,
        None,
        start.elapsed().as_micros() as u64,
    );

    // Call original creat
    let result = if let Some(original_fn) = get_original::<CreatFn>("creat") {
        original_fn(path, mode)
    } else {
        -1
    };

    let elapsed = start.elapsed().as_micros() as u64;
    if result >= 0 {
        log_syscall(
            "creat",
            vec![path_str, format!("{:o}", mode)],
            Some(format!("fd: {}", result)),
            None,
            elapsed,
        );
    } else {
        let error = std::io::Error::last_os_error().to_string();
        log_syscall(
            "creat",
            vec![path_str, format!("{:o}", mode)],
            Some(format!("failed: {}", result)),
            Some(error),
            elapsed,
        );
    }

    result
}

/// Intercept libc `unlink` to log file deletions.
///
/// # Safety
/// - `path` must be a valid, non-null C string pointer.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn unlink(path: *const c_char) -> c_int {
    let start = Instant::now();
    let path_str = c_str_to_string(path);

    // Log the attempt
    log_syscall(
        "unlink",
        vec![path_str.clone()],
        None,
        None,
        start.elapsed().as_micros() as u64,
    );

    // Call original unlink
    let result = if let Some(original_fn) = get_original::<UnlinkFn>("unlink") {
        original_fn(path)
    } else {
        -1
    };

    let elapsed = start.elapsed().as_micros() as u64;
    if result == 0 {
        log_syscall(
            "unlink",
            vec![path_str],
            Some("success".to_string()),
            None,
            elapsed,
        );
    } else {
        let error = std::io::Error::last_os_error().to_string();
        log_syscall(
            "unlink",
            vec![path_str],
            Some(format!("failed: {}", result)),
            Some(error),
            elapsed,
        );
    }

    result
}

/// Intercept libc `rename` to log file moves/renames.
///
/// # Safety
/// - `oldpath` and `newpath` must be valid, non-null C string pointers.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rename(oldpath: *const c_char, newpath: *const c_char) -> c_int {
    let start = Instant::now();
    let old_str = c_str_to_string(oldpath);
    let new_str = c_str_to_string(newpath);

    // Log the attempt
    log_syscall(
        "rename",
        vec![old_str.clone(), new_str.clone()],
        None,
        None,
        start.elapsed().as_micros() as u64,
    );

    // Call original rename
    let result = if let Some(original_fn) = get_original::<RenameFn>("rename") {
        original_fn(oldpath, newpath)
    } else {
        -1
    };

    let elapsed = start.elapsed().as_micros() as u64;
    if result == 0 {
        log_syscall(
            "rename",
            vec![old_str, new_str],
            Some("success".to_string()),
            None,
            elapsed,
        );
    } else {
        let error = std::io::Error::last_os_error().to_string();
        log_syscall(
            "rename",
            vec![old_str, new_str],
            Some(format!("failed: {}", result)),
            Some(error),
            elapsed,
        );
    }

    result
}
// duplicate openat removed; doc comments are above the original implementation
