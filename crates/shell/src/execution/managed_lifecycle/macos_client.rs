use anyhow::Result;
use serde_json::Value;
use substrate_common::{
    GuestPublisherPairingTicketV1, ManagedLifecyclePublisherRequestV1,
    PublisherBootstrapAuthorizationV1,
};

use super::provider_unavailable_error;

pub fn bootstrap_publisher_v1(_authorization: &PublisherBootstrapAuthorizationV1) -> Result<Value> {
    Err(provider_unavailable_error(
        "macos",
        "bootstrap_publisher_v1",
    ))
}

pub fn submit_publisher_request_v1(_request: &ManagedLifecyclePublisherRequestV1) -> Result<Value> {
    Err(provider_unavailable_error(
        "macos",
        "submit_publisher_request_v1",
    ))
}

pub fn issue_guest_publisher_pairing_ticket_v1(
    _request: &ManagedLifecyclePublisherRequestV1,
) -> Result<GuestPublisherPairingTicketV1> {
    Err(provider_unavailable_error(
        "macos",
        "issue_guest_publisher_pairing_ticket_v1",
    ))
}
