# Substrate Graph Database Integration

**Status**: Scaffolded (Phase 4 completion required)

This document describes the graph database integration for Substrate command tracing, enabling relationship analysis and security intelligence.

## Architectural Decision

**Problem**: Kuzu graph database integration was originally planned as a feature-gated module within `crates/trace`. However, analysis revealed:

- Multiple crates depend on `substrate-trace` for basic functionality (span creation, policy decisions)
- Kuzu is a heavy C++ dependency requiring cmake and 5-10 minute build times
- None of the dependent crates currently need graph functionality
- Future graph usage will expand beyond trace ingestion (CLI queries, security analysis, agent relationships)

**Solution**: Create dedicated `crates/substrate-graph` with clean separation:
- `substrate-trace` → lightweight core tracing (current users unaffected)
- `substrate-graph` → rich analysis features with heavy dependencies
- CLI/binaries → depend on both when graph features needed

## Current Implementation

### Basic Scaffold (Phase 4)
```rust
pub trait GraphDB: Send + Sync {
    fn init(&mut self, db_path: &Path) -> Result<()>;
    fn query(&self, query: &str) -> Result<Vec<Value>>;
    fn is_initialized(&self) -> bool;
}
```

### Feature Flags
- `kuzu-static`: Build Kuzu from source (requires cmake, slow build)
- `kuzu-dylib`: Link to system Kuzu library (fast build, requires system install)
- `mock`: Testing backend with no external dependencies

### Default Storage
- Database location: `~/.substrate/graph/`
- Privacy-aware: Configurable ignore patterns for sensitive paths

## Post-Phase 4 Implementation Plan

### Core Components
1. **GraphClient**: Main interface with backend abstraction
2. **KuzuBackend**: Kuzu database implementation
3. **MockBackend**: Testing implementation
4. **Ingestion Pipeline**: JSONL → Graph conversion
5. **Privacy Controls**: Selective indexing, hash-only mode
6. **Query Interface**: High-level typed queries

### Schema Design
```cypher
// Nodes
CREATE NODE TABLE Span(id STRING, cmd STRING, exit INT32, ts TIMESTAMP, PRIMARY KEY(id));
CREATE NODE TABLE File(path STRING, hash STRING, PRIMARY KEY(path));
CREATE NODE TABLE Agent(id STRING, type STRING, PRIMARY KEY(id));

// Relationships  
CREATE REL TABLE WROTE(FROM Span TO File, bytes INT);
CREATE REL TABLE READ(FROM Span TO File);
CREATE REL TABLE EXECUTED_BY(FROM Span TO Agent);
CREATE REL TABLE PARENT_OF(FROM Span TO Span);
```

### Integration Points
- **Trace Ingestion**: File-based tailer of `~/.substrate/trace.jsonl`
- **CLI Commands**: `substrate trace <span_id>` for graph queries
- **Privacy Integration**: Respect `~/.substrate/privacy.toml` settings
- **Performance Target**: <100ms for typical span queries

## Development Workflow

### Phase 4 (Current)
- [x] Architecture analysis complete
- [x] Basic crate scaffold created
- [x] Feature flags configured
- [x] Compilation verified

### Post-Phase 4
- [ ] Implement GraphClient and trait
- [ ] Add Kuzu backend implementation
- [ ] Create ingestion pipeline
- [ ] Add privacy controls
- [ ] Implement query interface
- [ ] CLI integration
- [ ] Comprehensive testing

## Build Instructions

```bash
# Default (no backend, compiles fast)
cargo build -p substrate-graph

# With mock backend (testing)
cargo build -p substrate-graph --features mock

# With Kuzu (requires cmake, slow build)
cargo build -p substrate-graph --features kuzu-static
```

## Expert Recommendations Applied

Based on deep architectural analysis:

1. **Clean Separation**: No types from substrate-graph appear in trace's public API
2. **Build Isolation**: Heavy dependencies only build when explicitly requested
3. **Backend Abstraction**: GraphDB trait enables testing and future backends
4. **Feature Matrix**: Multiple build options (static, dylib, mock)
5. **Migration Path**: Maintains compatibility with existing `~/.substrate/graph/` data

---

**Phase**: 4 (Scaffold Complete)  
**Next**: Full implementation after Phase 4 completion  
**Dependencies**: cmake (for Kuzu builds), system Kuzu library (for dylib builds)