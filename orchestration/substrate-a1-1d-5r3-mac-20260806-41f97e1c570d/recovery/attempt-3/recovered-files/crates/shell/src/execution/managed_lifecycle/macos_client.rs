use anyhow::{bail, Context, Result};
use serde_json::Value;
use substrate_common::{
    GuestPublisherPairingTicketV1, ManagedLifecyclePublisherRequestV1,
    PublisherBootstrapAuthorizationV1,
};

use super::provider_unavailable_error;

const MAC_MACH_SERVICE_V1: &str = "com.substrate.lifecycle.publisher.v1";
const MAC_CONTROL_DESIGNATED_REQUIREMENT_V1: &str =
    "anchor apple generic and identifier \"com.substrate.lifecycle.publisher.v1\"";

pub fn bootstrap_publisher_v1(authorization: &PublisherBootstrapAuthorizationV1) -> Result<Value> {
    let bytes = serde_json::to_vec(authorization).context("encode macOS publisher bootstrap")?;
    let response = open_mac_xpc_channel_v1("bootstrap-publisher", &bytes)?;
    serde_json::from_slice(&response).context("decode macOS publisher bootstrap response")
}

pub fn submit_publisher_request_v1(request: &ManagedLifecyclePublisherRequestV1) -> Result<Value> {
    let bytes = serde_json::to_vec(request).context("encode macOS publisher request")?;
    let response = open_mac_xpc_channel_v1("submit-request", &bytes)?;
    attest_mac_publisher_response_v1(&response)?;
    serde_json::from_slice(&response).context("decode macOS publisher response")
}

pub fn issue_guest_publisher_pairing_ticket_v1(
    request: &ManagedLifecyclePublisherRequestV1,
) -> Result<GuestPublisherPairingTicketV1> {
    let bytes = serde_json::to_vec(request).context("encode macOS pairing-ticket request")?;
    let response = open_mac_xpc_channel_v1("issue-guest-ticket", &bytes)?;
    attest_mac_publisher_response_v1(&response)?;
    serde_json::from_slice(&response).context("decode macOS pairing-ticket response")
}

/// Open exactly the fixed privileged Mach XPC service and return one bounded reply frame.
///
/// No executable, socket, environment value, or transport can be supplied by the caller. The
/// service validates the peer's code requirement before its handler accepts a publisher request.
pub fn open_mac_xpc_channel_v1(operation: &str, request: &[u8]) -> Result<Vec<u8>> {
    if !matches!(
        operation,
        "bootstrap-publisher" | "submit-request" | "issue-guest-ticket" | "lima-action"
    ) {
        bail!("unknown fixed macOS XPC publisher operation {operation}");
    }
    if request.is_empty() || request.len() > 1024 * 1024 {
        bail!("macOS XPC publisher request has an invalid frame size");
    }
    #[cfg(target_os = "macos")]
    unsafe {
        use std::ffi::CString;

        const XPC_CONNECTION_MACH_SERVICE_PRIVILEGED_V1: u64 = 1 << 1;
        let service = CString::new(MAC_MACH_SERVICE_V1).expect("fixed Mach service has no NUL");
        let operation = CString::new(operation).context("encode fixed macOS XPC operation")?;
        let connection = xpc_connection_create_mach_service(
            service.as_ptr(),
            std::ptr::null_mut(),
            XPC_CONNECTION_MACH_SERVICE_PRIVILEGED_V1,
        );
        if connection.is_null() {
            bail!("open fixed macOS XPC publisher service");
        }
        xpc_connection_activate(connection);
        let message = xpc_dictionary_create_empty();
        if message.is_null() {
            xpc_connection_cancel(connection);
            xpc_release(connection);
            bail!("create fixed macOS XPC publisher request");
        }
        xpc_dictionary_set_string(message, c"operation".as_ptr(), operation.as_ptr());
        xpc_dictionary_set_data(
            message,
            c"request".as_ptr(),
            request.as_ptr().cast(),
            request.len(),
        );
        let reply = xpc_connection_send_message_with_reply_sync(connection, message);
        xpc_release(message);
        xpc_connection_cancel(connection);
        xpc_release(connection);
        if reply.is_null() {
            bail!("fixed macOS XPC publisher returned no reply");
        }
        let mut length = 0usize;
        let bytes = xpc_dictionary_get_data(reply, c"response".as_ptr(), &mut length);
        if bytes.is_null() || length == 0 || length > 1024 * 1024 {
            xpc_release(reply);
            bail!("fixed macOS XPC publisher returned an invalid reply frame");
        }
        let response = std::slice::from_raw_parts(bytes.cast::<u8>(), length).to_vec();
        xpc_release(reply);
        Ok(response)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(provider_unavailable_error(
            "macos",
            "open_mac_xpc_channel_v1",
        ))
    }
}

/// Verify the fixed publisher service and XPC-enforced designated peer requirement in its reply.
pub fn attest_mac_publisher_response_v1(response: &[u8]) -> Result<()> {
    let value: Value =
        serde_json::from_slice(response).context("decode macOS publisher attestation")?;
    let attestation = value
        .get("xpc_attestation")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("macOS publisher response is missing xpc_attestation"))?;
    let service = attestation
        .get("mach_service")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("macOS publisher response is missing mach_service"))?;
    if service != MAC_MACH_SERVICE_V1 {
        bail!("macOS publisher response names an unexpected Mach service");
    }
    let requirement = attestation
        .get("peer_code_requirement")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow::anyhow!("macOS publisher response is missing peer_code_requirement")
        })?;
    if requirement != MAC_CONTROL_DESIGNATED_REQUIREMENT_V1 {
        bail!("macOS publisher response has an unexpected peer code requirement");
    }
    if attestation.get("audit_token").and_then(Value::as_str) != Some("xpc-verified") {
        bail!("macOS publisher response is missing XPC audit-token verification");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
#[link(name = "xpc")]
unsafe extern "C" {
    fn xpc_connection_create_mach_service(
        name: *const core::ffi::c_char,
        target_queue: *mut core::ffi::c_void,
        flags: u64,
    ) -> *mut core::ffi::c_void;
    fn xpc_connection_activate(connection: *mut core::ffi::c_void);
    fn xpc_connection_cancel(connection: *mut core::ffi::c_void);
    fn xpc_connection_send_message_with_reply_sync(
        connection: *mut core::ffi::c_void,
        message: *mut core::ffi::c_void,
    ) -> *mut core::ffi::c_void;
    fn xpc_dictionary_create_empty() -> *mut core::ffi::c_void;
    fn xpc_dictionary_set_string(
        dictionary: *mut core::ffi::c_void,
        key: *const core::ffi::c_char,
        value: *const core::ffi::c_char,
    );
    fn xpc_dictionary_set_data(
        dictionary: *mut core::ffi::c_void,
        key: *const core::ffi::c_char,
        bytes: *const core::ffi::c_void,
        length: usize,
    );
    fn xpc_dictionary_get_data(
        dictionary: *mut core::ffi::c_void,
        key: *const core::ffi::c_char,
        length: *mut usize,
    ) -> *const core::ffi::c_void;
    fn xpc_release(object: *mut core::ffi::c_void);
}
