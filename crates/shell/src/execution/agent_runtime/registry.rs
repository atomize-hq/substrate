use crate::execution::prompt_fulfillment::PromptFulfillmentBridge;

use super::validator::RuntimeSelectionDescriptor;

pub(crate) fn build_gateway_for_descriptor(
    descriptor: &RuntimeSelectionDescriptor,
) -> anyhow::Result<PromptFulfillmentBridge> {
    PromptFulfillmentBridge::for_descriptor(descriptor)
}
