# Substrate Gateway Policy Evaluation

This document is the durable canonical contract reference for Substrate gateway policy evaluation.
It mirrors the feature-local boundary contract and names the owned policy-evaluation surface without
expanding into a full decision matrix.

## Contract

The owned policy-evaluation surface is the gateway-integration flow over existing ADR-0027 inputs.

Owned contract points:
- the no-host-fallback rule when in-world execution is required
- the host-to-world secret delivery boundary
- the rule that gateway-local config, admin, and persistence surfaces are not trusted policy inputs
- the distinction between invalid integration state, dependency unavailability, and policy denial
- the rule that policy explanations remain Substrate-owned

Not defined here:
- a full decision table
- exhaustive routing examples
- implementation-specific transport or adapter behavior
- config schema ownership already covered elsewhere

## Boundary rules

- fail closed when in-world execution is required and no world boundary is available
- do not trust gateway-local control-plane surfaces as policy inputs
- host secret sourcing and delivery remain policy-gated and Substrate-owned
- policy evaluation remains part of the Substrate operator surface
