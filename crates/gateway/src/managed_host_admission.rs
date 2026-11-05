//! Ordinary managed Host route authentication. Startup metadata is not authority
//! to replace the service's held original/current policy or account admission.
use anyhow::{anyhow, ensure, Result};
use std::sync::Arc;

pub const ROUTE_INPUT_LEN: usize = 100;
pub const ROUTE_FD_ENV: &str = "SUBSTRATE_HOST_GATEWAY_ROUTE_FD";

/// Opaque syntax-only epoch/acquisition/route binding for sealed startup input.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct HostRouteBindingV1([u8; 64]);
impl HostRouteBindingV1 {
    /// Encode exact owner identifiers. This does not authenticate their issuer.
    // The binary also compiles this module; construction belongs to the service.
    #[allow(dead_code)]
    pub fn from_route(epoch: &str, acquisition: &str, handle: &str) -> Result<Self> {
        let route = handle
            .strip_prefix(&format!("hgl_{epoch}_"))
            .ok_or_else(|| anyhow!("managed Host route binding invalid"))?;
        let acquisition = acquisition
            .strip_prefix("hgt_")
            .ok_or_else(|| anyhow!("managed Host route binding invalid"))?;
        let mut value = [0; 64];
        decode_hex(epoch, &mut value[..16])?;
        decode_hex(acquisition, &mut value[16..48])?;
        decode_hex(route, &mut value[48..])?;
        Ok(Self(value))
    }
    /// Sealed startup metadata includes only a public verifier, never raw bearer.
    // The service uses the library copy; the binary only adopts inherited input.
    #[allow(dead_code)]
    pub fn startup_input(&self, verifier: &[u8; 32]) -> [u8; ROUTE_INPUT_LEN] {
        let mut input = [0; ROUTE_INPUT_LEN];
        input[..4].copy_from_slice(b"HRC1");
        input[4..68].copy_from_slice(&self.0);
        input[68..].copy_from_slice(verifier);
        input
    }
}
// Used by the library constructor and codec tests, not binary verification.
#[allow(dead_code)]
fn decode_hex(value: &str, destination: &mut [u8]) -> Result<()> {
    ensure!(
        value.len() == destination.len() * 2
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "managed Host route binding invalid"
    );
    for (index, byte) in destination.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| anyhow!("managed Host route binding invalid"))?;
    }
    Ok(())
}

pub(crate) struct ManagedHostRouteV1 {
    binding: HostRouteBindingV1,
    verifier: [u8; 32],
}
impl ManagedHostRouteV1 {
    pub(crate) fn adopt() -> Result<Arc<Self>> {
        use nix::libc;
        use std::io::Read;
        use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
        use std::os::unix::fs::MetadataExt;
        let raw =
            std::env::var(ROUTE_FD_ENV).map_err(|_| anyhow!("managed Host route FD missing"))?;
        std::env::remove_var(ROUTE_FD_ENV);
        let fd: i32 = raw
            .parse()
            .map_err(|_| anyhow!("managed Host route FD invalid"))?;
        ensure!(
            fd >= 3 && raw == fd.to_string(),
            "managed Host route FD invalid"
        );
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        ensure!(flags >= 0, "managed Host route FD unavailable");
        let owned = unsafe { OwnedFd::from_raw_fd(fd) };
        ensure!(
            unsafe { libc::fcntl(owned.as_raw_fd(), libc::F_SETFD, flags | libc::FD_CLOEXEC) } == 0,
            "managed Host route adoption failed"
        );
        let file = std::fs::File::from(owned);
        let metadata = file.metadata()?;
        let seals = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GET_SEALS) };
        let required =
            libc::F_SEAL_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL;
        ensure!(
            metadata.is_file()
                && metadata.uid() == 0
                && metadata.mode() & 0o7777 == 0o600
                && metadata.len() == ROUTE_INPUT_LEN as u64
                && seals >= 0
                && seals & required == required,
            "managed Host route input is not sealed service custody"
        );
        let mut input = Vec::new();
        file.take((ROUTE_INPUT_LEN + 1) as u64)
            .read_to_end(&mut input)?;
        ensure!(
            input.len() == ROUTE_INPUT_LEN && &input[..4] == b"HRC1",
            "managed Host route input invalid"
        );
        let mut binding = [0; 64];
        binding.copy_from_slice(&input[4..68]);
        let mut verifier = [0; 32];
        verifier.copy_from_slice(&input[68..]);
        Ok(Arc::new(Self {
            binding: HostRouteBindingV1(binding),
            verifier,
        }))
    }
    fn authorize(&self, headers: &mut axum::http::HeaderMap) -> bool {
        use base64::Engine;
        use sha2::{Digest, Sha256};
        let mut values = headers.get_all(axum::http::header::AUTHORIZATION).iter();
        let Some(value) = values.next() else {
            return false;
        };
        if values.next().is_some() {
            return false;
        }
        let Some(token) = value
            .to_str()
            .ok()
            .and_then(|value| value.strip_prefix("Bearer "))
        else {
            return false;
        };
        if token.len() != 43 {
            return false;
        }
        let Ok(decoded) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(token) else {
            return false;
        };
        if decoded.len() != 32
            || base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&decoded) != token
        {
            return false;
        }
        let mut hash = Sha256::new();
        hash.update(b"substrate.host-gateway.local-route.v1\0");
        hash.update(&self.binding.0[..48]);
        hash.update(decoded);
        let digest: [u8; 32] = hash.finalize().into();
        if digest != self.verifier {
            return false;
        }
        // No native local credential reaches providers, body traces or headers.
        headers.remove(axum::http::header::AUTHORIZATION);
        true
    }
}
pub(crate) async fn authenticate_native_request(
    axum::extract::State(route): axum::extract::State<Arc<ManagedHostRouteV1>>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    if !route.authorize(request.headers_mut()) {
        return axum::http::StatusCode::FORBIDDEN.into_response();
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_local_capability_is_bound_and_removed_without_granting_service_authority() {
        let epoch = "1".repeat(32);
        let key = format!("hgt_{}", "2".repeat(64));
        let handle = format!("hgl_{epoch}_{}", "3".repeat(32));
        let binding = HostRouteBindingV1::from_route(&epoch, &key, &handle).unwrap();
        let mut verifier = [0; 32];
        decode_hex(
            "a2abb73c94f9c5b5947c2014495f424bcc6b026927dcb35048780fad3dc535c2",
            &mut verifier,
        )
        .unwrap();
        let route = ManagedHostRouteV1 { binding, verifier };
        let mut headers = axum::http::HeaderMap::new();
        assert!(!route.authorize(&mut headers));
        headers.insert("authorization", "Bearer wrong".parse().unwrap());
        assert!(!route.authorize(&mut headers));
        headers.insert(
            "authorization",
            "Bearer AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                .parse()
                .unwrap(),
        );
        headers.append(
            "authorization",
            "Bearer AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                .parse()
                .unwrap(),
        );
        assert!(
            !route.authorize(&mut headers),
            "duplicate native credentials refuse"
        );
        headers.remove("authorization");
        headers.insert(
            "authorization",
            "Bearer AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                .parse()
                .unwrap(),
        );
        assert!(route.authorize(&mut headers));
        assert!(!headers.contains_key("authorization"));
        // Invalid local traffic cannot revoke the legitimate route.
        headers.insert(
            "authorization",
            "Bearer AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
                .parse()
                .unwrap(),
        );
        assert!(!route.authorize(&mut headers));
        headers.insert(
            "authorization",
            "Bearer AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                .parse()
                .unwrap(),
        );
        assert!(route.authorize(&mut headers));
        let other = ManagedHostRouteV1 {
            binding: HostRouteBindingV1::from_route(
                &"4".repeat(32),
                &key,
                &format!("hgl_{}_{}", "4".repeat(32), "3".repeat(32)),
            )
            .unwrap(),
            verifier,
        };
        headers.insert(
            "authorization",
            "Bearer AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                .parse()
                .unwrap(),
        );
        assert!(!other.authorize(&mut headers));
        assert_eq!(&binding.startup_input(&verifier)[..4], b"HRC1");
    }
}
