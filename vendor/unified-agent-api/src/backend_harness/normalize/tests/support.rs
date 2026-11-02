use std::{future::Future, pin::Pin};

use crate::backend_harness::test_support::{
    toy_kind, ToyBackendError, ToyCompletion, ToyEvent, ToyPolicy,
};
use crate::backend_harness::{
    BackendHarnessAdapter, BackendHarnessErrorPhase, BackendSpawn, NormalizedRequest,
};
use crate::{
    AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent, AgentWrapperKind,
    AgentWrapperRunRequest,
};

type ValidatePolicyFn = fn(&AgentWrapperRunRequest) -> Result<ToyPolicy, AgentWrapperError>;

fn validate_panic(_request: &AgentWrapperRunRequest) -> Result<ToyPolicy, AgentWrapperError> {
    panic!("validate_and_extract_policy must not be called for this test case");
}

fn validate_ok(_request: &AgentWrapperRunRequest) -> Result<ToyPolicy, AgentWrapperError> {
    Ok(ToyPolicy)
}

pub(super) struct PolicyFnAdapter {
    supported: &'static [&'static str],
    validate_fn: ValidatePolicyFn,
}

impl PolicyFnAdapter {
    pub(super) fn new(supported: &'static [&'static str], validate_fn: ValidatePolicyFn) -> Self {
        Self {
            supported,
            validate_fn,
        }
    }

    pub(super) fn panic_on_policy(supported: &'static [&'static str]) -> Self {
        Self::new(supported, validate_panic)
    }

    pub(super) fn ok_policy(supported: &'static [&'static str]) -> Self {
        Self::new(supported, validate_ok)
    }
}

impl BackendHarnessAdapter for PolicyFnAdapter {
    fn kind(&self) -> AgentWrapperKind {
        toy_kind()
    }

    fn supported_extension_keys(&self) -> &'static [&'static str] {
        self.supported
    }

    type Policy = ToyPolicy;

    fn validate_and_extract_policy(
        &self,
        request: &AgentWrapperRunRequest,
    ) -> Result<Self::Policy, AgentWrapperError> {
        (self.validate_fn)(request)
    }

    type BackendEvent = ToyEvent;
    type BackendCompletion = ToyCompletion;
    type BackendError = ToyBackendError;

    fn spawn(
        &self,
        _req: NormalizedRequest<Self::Policy>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        BackendSpawn<
                            Self::BackendEvent,
                            Self::BackendCompletion,
                            Self::BackendError,
                        >,
                        Self::BackendError,
                    >,
                > + Send
                + 'static,
        >,
    > {
        panic!("spawn must not be called from normalize_request");
    }

    fn map_event(&self, _event: Self::BackendEvent) -> Vec<AgentWrapperEvent> {
        panic!("map_event must not be called from normalize_request");
    }

    fn map_completion(
        &self,
        _completion: Self::BackendCompletion,
    ) -> Result<AgentWrapperCompletion, AgentWrapperError> {
        panic!("map_completion must not be called from normalize_request");
    }

    fn redact_error(&self, _phase: BackendHarnessErrorPhase, _err: &Self::BackendError) -> String {
        panic!("redact_error must not be called from normalize_request");
    }
}

pub(super) struct NeverCalledAdapter;

impl BackendHarnessAdapter for NeverCalledAdapter {
    fn kind(&self) -> AgentWrapperKind {
        panic!("kind must not be called for this test case");
    }

    fn supported_extension_keys(&self) -> &'static [&'static str] {
        panic!("supported_extension_keys must not be called for this test case");
    }

    type Policy = ToyPolicy;

    fn validate_and_extract_policy(
        &self,
        _request: &AgentWrapperRunRequest,
    ) -> Result<Self::Policy, AgentWrapperError> {
        panic!("validate_and_extract_policy must not be called for this test case");
    }

    type BackendEvent = ToyEvent;
    type BackendCompletion = ToyCompletion;
    type BackendError = ToyBackendError;

    fn spawn(
        &self,
        _req: NormalizedRequest<Self::Policy>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        BackendSpawn<
                            Self::BackendEvent,
                            Self::BackendCompletion,
                            Self::BackendError,
                        >,
                        Self::BackendError,
                    >,
                > + Send
                + 'static,
        >,
    > {
        panic!("spawn must not be called for this test case");
    }

    fn map_event(&self, _event: Self::BackendEvent) -> Vec<AgentWrapperEvent> {
        panic!("map_event must not be called for this test case");
    }

    fn map_completion(
        &self,
        _completion: Self::BackendCompletion,
    ) -> Result<AgentWrapperCompletion, AgentWrapperError> {
        panic!("map_completion must not be called for this test case");
    }

    fn redact_error(&self, _phase: BackendHarnessErrorPhase, _err: &Self::BackendError) -> String {
        panic!("redact_error must not be called for this test case");
    }
}
