use std::ffi::CString;
use std::mem;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::os::raw::c_int;
use std::time::Instant;

use crate::log_syscall;

// Type definitions for original functions
type ConnectFn = unsafe extern "C" fn(c_int, *const libc::sockaddr, libc::socklen_t) -> c_int;
type BindFn = unsafe extern "C" fn(c_int, *const libc::sockaddr, libc::socklen_t) -> c_int;
type AcceptFn = unsafe extern "C" fn(c_int, *mut libc::sockaddr, *mut libc::socklen_t) -> c_int;
type GetaddrInfoFn = unsafe extern "C" fn(
    *const libc::c_char,
    *const libc::c_char,
    *const libc::addrinfo,
    *mut *mut libc::addrinfo,
) -> c_int;

// Get original function pointer via dlsym
unsafe fn get_original<T>(name: &str) -> Option<T> {
    let c_name = CString::new(name).ok()?;
    let ptr = libc::dlsym(libc::RTLD_NEXT, c_name.as_ptr());
    if ptr.is_null() {
        None
    } else {
        Some(mem::transmute_copy(&ptr))
    }
}

// Convert sockaddr to string representation
unsafe fn sockaddr_to_string(addr: *const libc::sockaddr, len: libc::socklen_t) -> String {
    if addr.is_null() || len == 0 {
        return "null".to_string();
    }

    match (*addr).sa_family as i32 {
        libc::AF_INET if len >= mem::size_of::<libc::sockaddr_in>() as u32 => {
            let addr_in = addr as *const libc::sockaddr_in;
            let ip = Ipv4Addr::from(u32::from_be((*addr_in).sin_addr.s_addr));
            let port = u16::from_be((*addr_in).sin_port);
            format!("{}:{}", ip, port)
        }
        libc::AF_INET6 if len >= mem::size_of::<libc::sockaddr_in6>() as u32 => {
            let addr_in6 = addr as *const libc::sockaddr_in6;
            let ip = Ipv6Addr::from((*addr_in6).sin6_addr.s6_addr);
            let port = u16::from_be((*addr_in6).sin6_port);
            format!("[{}]:{}", ip, port)
        }
        libc::AF_UNIX => "unix_socket".to_string(),
        family => {
            format!("family_{}", family)
        }
    }
}

/// Intercept libc `connect` to log outbound connections.
///
/// # Safety
/// - `addr` must be a valid pointer to a `sockaddr` of length `addrlen`.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn connect(
    sockfd: c_int,
    addr: *const libc::sockaddr,
    addrlen: libc::socklen_t,
) -> c_int {
    let start = Instant::now();
    let addr_str = sockaddr_to_string(addr, addrlen);

    // Skip logging for local/unix sockets to reduce noise
    if addr_str == "unix_socket"
        || addr_str.starts_with("127.0.0.1:")
        || addr_str.starts_with("[::1]:")
    {
        if let Some(original_fn) = get_original::<ConnectFn>("connect") {
            return original_fn(sockfd, addr, addrlen);
        }
        return -1;
    }

    // Log the attempt
    log_syscall(
        "connect",
        vec![format!("fd:{}", sockfd), addr_str.clone()],
        None,
        None,
        start.elapsed().as_micros() as u64,
    );

    // Call original connect
    let result = if let Some(original_fn) = get_original::<ConnectFn>("connect") {
        original_fn(sockfd, addr, addrlen)
    } else {
        -1
    };

    let elapsed = start.elapsed().as_micros() as u64;
    if result == 0 {
        log_syscall(
            "connect",
            vec![format!("fd:{}", sockfd), addr_str],
            Some("success".to_string()),
            None,
            elapsed,
        );
    } else {
        let error = std::io::Error::last_os_error().to_string();
        log_syscall(
            "connect",
            vec![format!("fd:{}", sockfd), addr_str],
            Some(format!("failed: {}", result)),
            Some(error),
            elapsed,
        );
    }

    result
}

/// Intercept libc `bind` to log socket bindings.
///
/// # Safety
/// - `addr` must be a valid pointer to a `sockaddr` of length `addrlen`.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn bind(
    sockfd: c_int,
    addr: *const libc::sockaddr,
    addrlen: libc::socklen_t,
) -> c_int {
    let start = Instant::now();
    let addr_str = sockaddr_to_string(addr, addrlen);

    // Skip logging for unix sockets
    if addr_str == "unix_socket" {
        if let Some(original_fn) = get_original::<BindFn>("bind") {
            return original_fn(sockfd, addr, addrlen);
        }
        return -1;
    }

    // Log the attempt
    log_syscall(
        "bind",
        vec![format!("fd:{}", sockfd), addr_str.clone()],
        None,
        None,
        start.elapsed().as_micros() as u64,
    );

    // Call original bind
    let result = if let Some(original_fn) = get_original::<BindFn>("bind") {
        original_fn(sockfd, addr, addrlen)
    } else {
        -1
    };

    let elapsed = start.elapsed().as_micros() as u64;
    if result == 0 {
        log_syscall(
            "bind",
            vec![format!("fd:{}", sockfd), addr_str],
            Some("success".to_string()),
            None,
            elapsed,
        );
    } else {
        let error = std::io::Error::last_os_error().to_string();
        log_syscall(
            "bind",
            vec![format!("fd:{}", sockfd), addr_str],
            Some(format!("failed: {}", result)),
            Some(error),
            elapsed,
        );
    }

    result
}

/// Intercept libc `accept` to log inbound connections.
///
/// # Safety
/// - `addr` and `addrlen` must be valid pointers if non-null.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn accept(
    sockfd: c_int,
    addr: *mut libc::sockaddr,
    addrlen: *mut libc::socklen_t,
) -> c_int {
    let start = Instant::now();

    // Call original accept
    let result = if let Some(original_fn) = get_original::<AcceptFn>("accept") {
        original_fn(sockfd, addr, addrlen)
    } else {
        -1
    };

    let elapsed = start.elapsed().as_micros() as u64;

    if result >= 0 {
        let client_addr = if !addr.is_null() && !addrlen.is_null() {
            sockaddr_to_string(addr, *addrlen)
        } else {
            "unknown".to_string()
        };

        // Skip logging for unix sockets and localhost
        if client_addr != "unix_socket"
            && !client_addr.starts_with("127.0.0.1:")
            && !client_addr.starts_with("[::1]:")
        {
            log_syscall(
                "accept",
                vec![format!("fd:{}", sockfd), client_addr],
                Some(format!("new_fd: {}", result)),
                None,
                elapsed,
            );
        }
    } else {
        let error = std::io::Error::last_os_error().to_string();
        log_syscall(
            "accept",
            vec![format!("fd:{}", sockfd)],
            Some(format!("failed: {}", result)),
            Some(error),
            elapsed,
        );
    }

    result
}

/// Intercept libc `getaddrinfo` for DNS resolution tracking.
///
/// # Safety
/// - `node`, `service`, `hints`, and `res` must be valid according to libc contract.
/// - This function forwards to the original libc implementation and thus inherits its safety requirements.
#[no_mangle]
pub unsafe extern "C" fn getaddrinfo(
    node: *const libc::c_char,
    service: *const libc::c_char,
    hints: *const libc::addrinfo,
    res: *mut *mut libc::addrinfo,
) -> c_int {
    let start = Instant::now();

    let node_str = if !node.is_null() {
        std::ffi::CStr::from_ptr(node)
            .to_string_lossy()
            .into_owned()
    } else {
        "null".to_string()
    };

    let service_str = if !service.is_null() {
        std::ffi::CStr::from_ptr(service)
            .to_string_lossy()
            .into_owned()
    } else {
        "null".to_string()
    };

    // Skip common local lookups
    if node_str == "localhost" || node_str == "127.0.0.1" || node_str == "::1" {
        if let Some(original_fn) = get_original::<GetaddrInfoFn>("getaddrinfo") {
            return original_fn(node, service, hints, res);
        }
        return -1;
    }

    // Log the DNS lookup
    log_syscall(
        "getaddrinfo",
        vec![node_str.clone(), service_str.clone()],
        None,
        None,
        start.elapsed().as_micros() as u64,
    );

    // Call original getaddrinfo
    let result = if let Some(original_fn) = get_original::<GetaddrInfoFn>("getaddrinfo") {
        original_fn(node, service, hints, res)
    } else {
        libc::EAI_SYSTEM
    };

    let elapsed = start.elapsed().as_micros() as u64;

    if result == 0 {
        // Extract resolved addresses
        let mut addresses = Vec::new();
        if !res.is_null() && !(*res).is_null() {
            let mut current = *res;
            while !current.is_null() {
                if !(*current).ai_addr.is_null() {
                    let addr_str = sockaddr_to_string((*current).ai_addr, (*current).ai_addrlen);
                    addresses.push(addr_str);
                }
                current = (*current).ai_next;
            }
        }

        log_syscall(
            "getaddrinfo",
            vec![node_str, service_str],
            Some(format!("resolved: {:?}", addresses)),
            None,
            elapsed,
        );
    } else {
        log_syscall(
            "getaddrinfo",
            vec![node_str, service_str],
            Some(format!("failed: {}", result)),
            Some(format!("error_code: {}", result)),
            elapsed,
        );
    }

    result
}
