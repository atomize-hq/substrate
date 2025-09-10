use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::time::Instant;

use crate::{c_str_array_to_vec, c_str_to_string, inherit_correlation_env, log_syscall};

// Type for the original function pointers
type ExecveFn =
    unsafe extern "C" fn(*const c_char, *const *const c_char, *const *const c_char) -> c_int;
type ExecvpFn = unsafe extern "C" fn(*const c_char, *const *const c_char) -> c_int;
type SystemFn = unsafe extern "C" fn(*const c_char) -> c_int;
type PopenFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut libc::FILE;

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

/// Intercept libc `execve` to inject correlation environment variables and log the call.
///
/// # Safety
/// - `path` must be a valid, non-null C string pointer.
/// - `argv` and `envp` must be valid, null-terminated arrays of C string pointers.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn execve(
    path: *const c_char,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> c_int {
    let start = Instant::now();
    let path_str = c_str_to_string(path);
    let args = c_str_array_to_vec(argv);

    // Log the attempt
    log_syscall(
        "execve",
        vec![path_str.clone(), format!("{:?}", args)],
        None,
        None,
        start.elapsed().as_micros() as u64,
    );

    // Inject correlation environment variables
    let mut new_env = Vec::new();
    if !envp.is_null() {
        // Copy existing environment
        let mut i = 0;
        loop {
            let env_ptr = *envp.offset(i);
            if env_ptr.is_null() {
                break;
            }
            new_env.push(env_ptr);
            i += 1;
        }
    }

    // Add our correlation vars
    let correlation_vars = inherit_correlation_env();
    let mut c_strings: Vec<CString> = Vec::new();
    for (key, value) in correlation_vars {
        if let Ok(c_str) = CString::new(format!("{}={}", key, value)) {
            c_strings.push(c_str);
        }
    }

    // Convert to pointers
    for c_str in &c_strings {
        new_env.push(c_str.as_ptr());
    }
    new_env.push(ptr::null());

    // Call original execve
    if let Some(original_fn) = get_original::<ExecveFn>("execve") {
        let result = original_fn(path, argv, new_env.as_ptr());

        // Only reached if execve fails
        let elapsed = start.elapsed().as_micros() as u64;
        let error = std::io::Error::last_os_error().to_string();
        log_syscall(
            "execve",
            vec![path_str, format!("{:?}", args)],
            Some(format!("failed: {}", result)),
            Some(error),
            elapsed,
        );

        result
    } else {
        libc::ENOSYS // Function not found
    }
}

/// Intercept libc `execvp` to inject correlation environment variables and log the call.
///
/// # Safety
/// - `file` must be a valid, non-null C string pointer.
/// - `argv` must be a valid, null-terminated array of C string pointers.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn execvp(file: *const c_char, argv: *const *const c_char) -> c_int {
    let start = Instant::now();
    let file_str = c_str_to_string(file);
    let args = c_str_array_to_vec(argv);

    // Log the attempt
    log_syscall(
        "execvp",
        vec![file_str.clone(), format!("{:?}", args)],
        None,
        None,
        start.elapsed().as_micros() as u64,
    );

    // Need to construct envp with our variables
    // execvp uses current environment, so we modify it
    for (key, value) in inherit_correlation_env() {
        std::env::set_var(key, value);
    }

    // Call original execvp
    if let Some(original_fn) = get_original::<ExecvpFn>("execvp") {
        let result = original_fn(file, argv);

        // Only reached if execvp fails
        let elapsed = start.elapsed().as_micros() as u64;
        let error = std::io::Error::last_os_error().to_string();
        log_syscall(
            "execvp",
            vec![file_str, format!("{:?}", args)],
            Some(format!("failed: {}", result)),
            Some(error),
            elapsed,
        );

        result
    } else {
        libc::ENOSYS
    }
}

/// Intercept libc `system` to inject correlation environment variables and log the call.
///
/// # Safety
/// - `command` must be a valid, non-null C string pointer.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn system(command: *const c_char) -> c_int {
    let start = Instant::now();
    let cmd_str = c_str_to_string(command);

    // Log the attempt
    log_syscall(
        "system",
        vec![cmd_str.clone()],
        None,
        None,
        start.elapsed().as_micros() as u64,
    );

    // Set correlation environment
    for (key, value) in inherit_correlation_env() {
        std::env::set_var(key, value);
    }

    // Call original system
    if let Some(original_fn) = get_original::<SystemFn>("system") {
        let result = original_fn(command);

        let elapsed = start.elapsed().as_micros() as u64;
        log_syscall(
            "system",
            vec![cmd_str],
            Some(format!("exit: {}", result)),
            None,
            elapsed,
        );

        result
    } else {
        -1
    }
}

/// Intercept libc `popen` to inject correlation environment variables and log the call.
///
/// # Safety
/// - `command` and `mode` must be valid, non-null C string pointers.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn popen(command: *const c_char, mode: *const c_char) -> *mut libc::FILE {
    let start = Instant::now();
    let cmd_str = c_str_to_string(command);
    let mode_str = c_str_to_string(mode);

    // Log the attempt
    log_syscall(
        "popen",
        vec![cmd_str.clone(), mode_str.clone()],
        None,
        None,
        start.elapsed().as_micros() as u64,
    );

    // Set correlation environment
    for (key, value) in inherit_correlation_env() {
        std::env::set_var(key, value);
    }

    // Call original popen
    if let Some(original_fn) = get_original::<PopenFn>("popen") {
        let result = original_fn(command, mode);

        let elapsed = start.elapsed().as_micros() as u64;
        let success = !result.is_null();
        log_syscall(
            "popen",
            vec![cmd_str, mode_str],
            Some(format!("success: {}", success)),
            if success {
                None
            } else {
                Some("NULL FILE*".to_string())
            },
            elapsed,
        );

        result
    } else {
        ptr::null_mut()
    }
}
