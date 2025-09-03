# Phase 4 Continuation Prompt

## Current Status Summary
You are continuing Phase 4 implementation of the Substrate project. **Major progress has been made** - the core infrastructure is now working.

### ✅ Recently Completed (Session 2025-09-02)
1. **Agent API Compilation Fixed** - All 5 crates now compile successfully
2. **Shim Span Generation** - Trace spans now created for all intercepted commands  
3. **Integration Verified** - Shell + Shim + Broker + Trace all working together

### 📁 Key Documents to Reference
1. **Master Plan**: `docs/project_management/future/implementation_phase4_merged.md`
2. **Current Progress**: `docs/project_management/future/PHASE_4_PROGRESS.md` 
3. **Trace Documentation**: `TRACE.md`
4. **Broker Documentation**: `BROKER.md`
5. **Project Instructions**: `CLAUDE.md`

### 🎯 Current Working Directory
```
/Users/spensermcconnell/__Active_Code/atomize-hq/substrate
```

## Immediate Next Priority: PR#10

According to the master plan (`implementation_phase4_merged.md`), you should implement:

### 1. Overlayfs Filesystem Diff Tracking
**Goal**: Populate the `fs_diff` field in trace spans with actual filesystem changes.

**Files to Create/Modify**:
- `crates/world/src/overlayfs.rs` - New module for filesystem diff tracking
- `crates/trace/src/lib.rs` - Update to receive and store fs_diff data
- Integration points in shell and shim to collect filesystem changes

**Current State**: 
- Trace spans show `"fs_diff": null` (placeholder)
- Need to implement actual diff tracking using overlayfs or similar

### 2. Network Filtering with nftables  
**Goal**: Populate the `scopes_used` field with actual network access tracking.

**Files to Create/Modify**:
- `crates/world/src/netfilter.rs` - New module for network filtering
- Integration with world backend to track network access
- Update trace collection to capture network scopes

**Current State**:
- Trace spans show `"scopes_used": []` (placeholder) 
- Need real network filtering and scope tracking

## Architecture Context

### Current Integration Flow
```
User Command → Shell (creates span) → Broker (policy check) → Shim (creates span) → Execute → Complete spans
                                                                                                    ↓
                                                                                            trace.jsonl
```

### What's Working Now
- ✅ Policy evaluation and decisions captured in spans
- ✅ Command execution tracing end-to-end
- ✅ Exit codes and timing recorded
- ✅ Agent API compiles and ready for runtime testing
- ✅ Unix socket communication architecture established

### What Needs Implementation  
- ❌ Real filesystem diff tracking (currently placeholder)
- ❌ Real network scope tracking (currently placeholder)
- ❌ Agent API runtime testing in actual VM environment
- ❌ LD_PRELOAD syscall interception (PR#11)

## Development Commands

```bash
# Verify current state works
SUBSTRATE_WORLD=enabled cargo run --bin substrate -- -c "echo test"
tail -2 ~/.substrate/trace.jsonl | jq .

# Build all components
cargo build --workspace

# Run tests
cargo test -p substrate-trace
cargo test -p substrate-broker

# Build specific components for PR#10
cargo build -p world -p substrate-trace
```

## Testing Verification

Before implementing new features, verify current state:
1. Run `SUBSTRATE_WORLD=enabled` commands and check spans are generated
2. Verify policy evaluation works with broker
3. Confirm all Agent API crates build without errors
4. Check trace.jsonl contains proper span data

## Key Implementation Notes

1. **Preserve Existing Functionality**: All Phase 1-3 features must continue working
2. **Performance Budget**: Keep overhead < 50ms per command
3. **Security First**: All code should be secure by default
4. **Follow Patterns**: Use existing trace and broker integration patterns
5. **Test Coverage**: Add tests for new functionality

## Expected Challenges for PR#10

1. **Overlayfs Complexity**: Filesystem diff tracking requires careful handling of:
   - Mount points and namespaces
   - File permissions and ownership
   - Large file changes (need efficient diffing)
   
2. **Network Filtering**: nftables integration needs:
   - Root privileges or capability handling  
   - Rule management without conflicts
   - Clean cleanup on exit

3. **Integration Points**: Both features need to integrate with:
   - World backend architecture
   - Trace span generation
   - Policy enforcement

## Success Criteria for PR#10

- [ ] `fs_diff` field in traces shows actual filesystem changes
- [ ] `scopes_used` field shows actual network access
- [ ] No performance regression (< 50ms overhead maintained)
- [ ] All existing tests continue to pass
- [ ] New tests added for filesystem and network tracking
- [ ] Documentation updated for new features

## After PR#10 Completion

The next priorities will be:
- **PR#11**: LD_PRELOAD telemetry library for syscall interception
- **PR#12**: HRM scaffolding for trace replay
- **PR#13**: Kuzu graph database integration  
- **PR#14**: Comprehensive test suites

## Getting Started Commands

```bash
# 1. First, verify current state
cd /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
SUBSTRATE_WORLD=enabled cargo run --bin substrate -- -c "ls -la"

# 2. Check what was traced
tail -4 ~/.substrate/trace.jsonl | jq .

# 3. Review the master plan
cat docs/project_management/future/implementation_phase4_merged.md

# 4. Start implementing overlayfs or network filtering
# (Choose based on complexity and dependencies)
```

## Important Reminders

1. **Read the Progress Doc First**: `docs/project_management/future/PHASE_4_PROGRESS.md` has detailed context
2. **Unix Socket is Critical**: Don't change the `/run/substrate.sock` architecture - it's part of the VM communication design
3. **Trace Integration**: Use existing `create_span_builder()` patterns for consistency
4. **Policy Integration**: Leverage existing broker integration patterns
5. **Test Early and Often**: Verify each component as you build it

The foundation is solid - now it's time to build the advanced features on top of it. Start with the area you're most confident in (filesystem or network) and build incrementally.

---
**Created**: 2025-09-02  
**Phase**: 4 (Active Development)  
**Priority**: PR#10 - Overlayfs & Network Filtering