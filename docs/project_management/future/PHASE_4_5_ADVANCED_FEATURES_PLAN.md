# Phase 4.5: Advanced Features Implementation Plan

## Overview

Phase 4.5 completes the advanced features deferred from Phase 4: concurrent agent output and comprehensive graph intelligence. While Phase 4 successfully implemented all core infrastructure, these sophisticated features require dedicated focus to implement properly.

**Dual Objectives**:
1. **Concurrent Agent Output**: Zero-overhead async REPL with `tokio::select!` for real-time agent communication  
2. **Graph Intelligence**: Complete Kuzu-based graph database with plugin architecture for command relationship analysis

**Strategic Rationale**: Both features are critical for the future product vision but were correctly deferred to maintain Phase 4's core functionality timeline.

---

## Related Design Docs

- Isolation Upgrade (Phase 4.5): See PHASE_4_5_ISOLATION_UPGRADE.md for a focused plan covering netns‑by‑default networking isolation, scoped nft policy, and PID fidelity across strategies.

---

## Part A: Concurrent Agent Output Implementation (Path A Now)

### Background & Problem Statement

#### Issue with Original Approach
During Phase 4, the ExternalPrinter implementation for concurrent output caused **2.4% idle CPU usage** due to polling every 100ms, even when no messages were being sent.

**Root Cause**: 
- ExternalPrinter uses `event::poll()` with 100ms timeout
- Results in 10 polls/second, each consuming ~2.4ms of CPU time
- Constant 2.4% CPU usage even when completely idle

**Resolution**: ExternalPrinter was removed completely, eliminating the CPU waste.

### Current State
- **Standard sync REPL**: Working perfectly with 0% idle CPU usage
- **Agent API Infrastructure**: Complete and ready for integration
- **Communication Channels**: host-proxy ↔ world-agent architecture operational

### Path A: Non‑Polling Renderer with Reedline

We will keep the existing Reedline REPL and add a background renderer thread that blocks on a channel and renders agent/events without polling. This achieves zero idle CPU and minimal change risk.

#### Components
- `AgentHub` (see Appendix A): central registry + event bus where agents dock and publish events.
- `EventRenderer` (new, inside shell): reads events via blocking recv, uses `Reedline::suspend_guard()` to safely draw output and restore the prompt.

#### Event types (minimal)
```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AgentEventKind { Registered, Status, TaskStart, TaskProgress, TaskEnd, PtyData, Alert }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentEvent {
  pub ts: chrono::DateTime<chrono::Utc>,
  pub agent_id: String,
  pub project: Option<String>,
  pub kind: AgentEventKind,
  pub data: serde_json::Value,
}
```

#### Shell integration (code sketch)
```rust
// crates/shell/src/lib.rs
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

fn start_event_renderer(mut rx: UnboundedReceiver<AgentEvent>, line_editor: &mut Reedline) {
    let handle = std::thread::spawn(move || {
        while let Some(evt) = rx.blocking_recv() {
            // Temporarily suspend prompt rendering
            // (in practice, pass a lightweight channel or use a global to reach the editor)
            // Here we assume a global editor guard function is available.
            redraw_suspended(|out| {
                // Minimal formatted line per event
                use std::io::Write;
                writeln!(out, "[{}] {}", evt.agent_id, evt.kind_string()).ok();
            });
        }
    });
    // store handle to join on shutdown
}
```

#### Agent status panel (minimal)
- Maintain a `HashMap<agent_id, Status>` updated by events; a `substrate agents list` command prints a summarized table (active/dormant, last event ts).

CLI changes (shell)
- Extend `Cli` with a new top‑level subcommand `agents`:
```rust
#[derive(clap::Subcommand, Debug)]
pub enum AgentsCmd { List }

#[derive(clap::Subcommand, Debug)]
pub enum GraphCmd {
  Ingest { file: std::path::PathBuf },
  Status,
  WhatChanged { span_id: String, #[arg(long, default_value_t = 100)] limit: usize },
}

#[derive(Parser, Debug)]
pub struct Cli {
  // existing flags
  #[command(subcommand)]
  pub sub: Option<SubCommands>,
}

#[derive(clap::Subcommand, Debug)]
pub enum SubCommands { Agents(AgentsCmd), Graph(GraphCmd) }
```
- In `run_shell()`, if `sub` is Some, dispatch to `handle_agents_list()` or graph handlers and exit.
- `handle_agents_list()` calls Hub `GET /v1/agents` and renders a table (id, state, project, last_seen).

#### Acceptance criteria
- Idle CPU < 0.1% with agents connected but silent.
- Concurrent PTY data and user typing do not corrupt the prompt; output appears above the prompt.
- Works on macOS/Linux/Windows terminals.

#### Tests
- Unit: event formatting and status transitions.
- Manual: run shell, start two agents emitting events, verify no prompt corruption and no idle CPU spin.

---

## Part B: Graph Intelligence Implementation

### Vision & Strategic Importance

Graph intelligence is **critical for the future product** as it enables:
- **Command Relationship Analysis**: Understand how commands relate and depend on each other
- **Security Pattern Detection**: Identify suspicious command sequences and data flows
- **Dependency Visualization**: Map file, network, and process relationships
- **Agent Behavior Analysis**: Track AI agent interaction patterns over time
- **Regression Analysis**: Compare current vs historical command patterns

### Current State Analysis

#### What Exists (Phase 4 Scaffold)
```rust
// Current substrate-graph/src/lib.rs (minimal)
pub trait GraphDB: Send + Sync {
    fn init(&mut self, db_path: &Path) -> Result<()>;
    fn query(&self, query: &str) -> Result<Vec<Value>>;
    fn is_initialized(&self) -> bool;
}

pub enum Backend {
    #[cfg(feature = "kuzu-static")] Kuzu,
    #[cfg(feature = "mock")] Mock,
}
```

#### What's Missing (85% of functionality)
- Actual Kuzu backend implementation
- Graph schema design (nodes, edges, relationships)
- JSONL trace ingestion pipeline  
- High-level query interface
- Privacy controls and data filtering
- Performance optimization for large graphs

### Plugin Architecture Design (Compile‑Time Backends)

#### Strategic Requirements

1. **Kuzu as Default**: Built-in, optimized Kuzu integration for 90% of users
2. **Extensible Design**: Users can add custom graph backends later
3. **Zero-Overhead**: When graph is disabled, zero performance impact
4. **Future-Proof**: Architecture supports advanced graph databases
5. **Developer-Friendly**: Simple to add new backends

#### Architectural Approaches Analysis

##### Option A: Compile-Time Plugin Architecture (Recommended)
**Pattern**: Feature-flag based backend selection with trait objects
**Used by**: reqwest, tokio, database ORMs

```rust
// Core trait in substrate-graph-core
#[async_trait]
pub trait GraphDB: Send + Sync {
    async fn new(config: &GraphConfig) -> Result<Self, GraphDbError> 
    where Self: Sized;
    
    async fn query(&self, query: &GraphQuery) -> Result<QueryResult, GraphDbError>;
    
    async fn ingest_span(&self, span: &TraceSpan) -> Result<(), GraphDbError>;
    
    async fn batch_ingest(&self, spans: Vec<TraceSpan>) -> Result<(), GraphDbError>;
}

// Factory function
pub fn create_graph_db(config: &GraphConfig) -> Result<Box<dyn GraphDB>, GraphDbError> {
    match config.backend_type.as_str() {
        #[cfg(feature = "backend-kuzu")]
        "kuzu" => Ok(Box::new(KuzuBackend::new(config).await?)),
        
        #[cfg(feature = "backend-neo4j")]  
        "neo4j" => Ok(Box::new(Neo4jBackend::new(config).await?)),
        
        #[cfg(feature = "backend-mock")]
        "mock" => Ok(Box::new(MockBackend::new(config).await?)),
        
        _ => Err(GraphDbError::UnsupportedBackend(config.backend_type.clone())),
    }
}
```

**Benefits**:
- ✅ **Maximum Safety**: Full static analysis and Rust safety guarantees
- ✅ **Zero Runtime Overhead**: Static dispatch when possible  
- ✅ **Lean Binaries**: Only compile backends actually used
- ✅ **Simple Dependencies**: Clear dependency tree, no FFI complexity
- ✅ **Rust Ecosystem Standard**: Familiar pattern to Rust developers

**Trade-offs**:
- ⚠️ **Requires Recompilation**: Adding new backend needs rebuild
- ⚠️ **Compile-Time Selection**: Can't change backends at runtime

##### Option B: Dynamic Loading Plugin Architecture  
**Pattern**: Runtime shared library loading with stable ABI
**Used by**: High-flexibility systems requiring hot-swapping

```rust
// Using abi_stable crate for FFI-safe interfaces
use abi_stable::{StableAbi, sabi_trait};

#[sabi_trait]
pub trait GraphDBPlugin: Send + Sync {
    fn init(&mut self, config: RString) -> RResult<(), GraphDbError>;
    fn query(&self, query: RString) -> RResult<RVec<RString>, GraphDbError>;
}

// Plugin loading
pub struct GraphPluginManager {
    plugins: Vec<DynTrait<'static, RBox<()>, GraphDBPlugin_TO<'static>>>,
}

impl GraphPluginManager {
    pub fn load_plugin(&mut self, path: &str) -> Result<(), PluginError> {
        let lib = abi_stable::library::lib_header_from_path(path)?;
        let plugin = lib.init_root_module::<GraphPlugin_Ref>()?;
        self.plugins.push(plugin);
        Ok(())
    }
}
```

**Benefits**:
- ✅ **Ultimate Flexibility**: Add backends without recompilation
- ✅ **Hot-Swapping**: Change backends at runtime
- ✅ **Third-Party Extensions**: External developers can create backends

**Trade-offs**:
- ❌ **Massive Complexity**: Stable ABI, unsafe FFI, error propagation challenges  
- ❌ **Performance Overhead**: ~2x slower than static dispatch
- ❌ **Safety Risks**: Plugin bugs can crash entire system
- ❌ **Platform-Specific**: Different shared library formats (.so, .dll, .dylib)

##### Option C: Hybrid Architecture (Advanced)
**Pattern**: Default compile-time + optional runtime plugins
**Used by**: Complex systems with both built-in and extensible backends

```rust
pub enum GraphBackend {
    // Compile-time backends (safe, fast)
    #[cfg(feature = "backend-kuzu")]
    Kuzu(KuzuBackend),
    
    #[cfg(feature = "backend-mock")]
    Mock(MockBackend),
    
    // Runtime plugins (flexible, slower)
    #[cfg(feature = "dynamic-plugins")]
    Plugin(Box<dyn GraphDBPlugin>),
}
```

**Benefits**:
- ✅ **Best of Both**: Fast built-ins + extensible plugins
- ✅ **Migration Path**: Start with compile-time, add dynamic later

**Trade-offs**:
- ❌ **High Complexity**: Must maintain both architectures
- ⚠️ **Feature Creep Risk**: Easy to over-engineer

### Recommended Architecture: Compile-Time Plugin System

**Decision**: **Option A - Compile-Time Plugin Architecture**

**Rationale**:
1. **Target Audience**: Developer tool where recompilation is acceptable  
2. **Safety Critical**: Command tracing security can't tolerate plugin crashes
3. **Performance Priority**: Graph queries must be <100ms for good UX
4. **Simplicity**: Easier to maintain, debug, and extend
5. **Future Path**: Can add Option C hybrid later if needed

### Graph Schema Design

#### Core Entities
```cypher
// Command Execution Nodes
CREATE NODE TABLE Command(
    span_id STRING,
    session_id STRING, 
    cmd STRING,
    args STRING[], 
    exit_code INT64,
    duration_ms INT64,
    component STRING, // shell, shim, telemetry
    cwd STRING,
    timestamp TIMESTAMP,
    PRIMARY KEY (span_id)
);

// File System Nodes  
CREATE NODE TABLE File(
    path STRING,
    type STRING, // file, directory, symlink
    permissions STRING,
    size INT64,
    PRIMARY KEY (path)
);

// Network Nodes
CREATE NODE TABLE NetworkEndpoint(
    address STRING, // IP:port or domain:port
    resolved_ip STRING,
    scope STRING, // internal, external, blocked
    PRIMARY KEY (address)
);

// Agent Nodes
CREATE NODE TABLE Agent(
    agent_id STRING,
    agent_type STRING, // claude, gpt, custom
    session_id STRING,
    capabilities STRING[],
    PRIMARY KEY (agent_id)
);

// Policy Nodes
CREATE NODE TABLE Policy(
    policy_id STRING,
    name STRING,
    rules STRING, // YAML content hash for change tracking
    active_from TIMESTAMP,
    PRIMARY KEY (policy_id)
);
```

#### Relationship Types
```cypher
// Command Relationships
CREATE REL TABLE Executes(FROM Agent TO Command, agent_context STRING);
CREATE REL TABLE ChildOf(FROM Command TO Command, fork_type STRING);
CREATE REL TABLE TriggeredBy(FROM Command TO Command, trigger_type STRING);

// File Relationships  
CREATE REL TABLE Reads(FROM Command TO File, bytes_read INT64, access_time TIMESTAMP);
CREATE REL TABLE Writes(FROM Command TO File, bytes_written INT64, content_hash STRING);
CREATE REL TABLE Modifies(FROM Command TO File, change_type STRING, before_size INT64, after_size INT64);
CREATE REL TABLE Deletes(FROM Command TO File, deletion_time TIMESTAMP);

// Network Relationships
CREATE REL TABLE Connects(FROM Command TO NetworkEndpoint, protocol STRING, bytes_sent INT64, bytes_received INT64);

// Policy Relationships
CREATE REL TABLE EnforcedBy(FROM Command TO Policy, decision STRING, approval_required BOOL);
CREATE REL TABLE Violates(FROM Command TO Policy, violation_type STRING, severity STRING);
```

### High-Level Use Cases & Queries

#### 1. Security Analysis Queries
```rust
// Find commands that accessed sensitive files
pub async fn find_sensitive_access(db: &dyn GraphDB) -> Result<Vec<SecurityAlert>> {
    let query = r#"
        MATCH (c:Command)-[r:Reads|Writes]->(f:File)
        WHERE f.path CONTAINS '/etc/' OR f.path CONTAINS '.ssh/' OR f.path CONTAINS '.aws/'
        RETURN c.span_id, c.cmd, f.path, r.access_time
        ORDER BY r.access_time DESC
        LIMIT 100
    "#;
    
    let results = db.query(query).await?;
    // Convert to SecurityAlert structs...
}

// Detect unusual command sequences
pub async fn detect_anomalies(db: &dyn GraphDB, session_id: &str) -> Result<Vec<Anomaly>> {
    let query = r#"
        MATCH (c1:Command)-[:ChildOf]->(c2:Command)
        WHERE c1.session_id = $session_id 
        AND c1.cmd CONTAINS 'curl' AND c2.cmd CONTAINS 'bash'
        RETURN c1, c2
    "#;
    
    let results = db.query(query).await?;
    // Analyze for security patterns...
}
```

#### 2. Development Intelligence Queries  
```rust
// What files did this command change?
pub async fn what_changed(db: &dyn GraphDB, span_id: &str) -> Result<Vec<FileChange>> {
    let query = r#"
        MATCH (c:Command {span_id: $span_id})-[r:Writes|Modifies|Deletes]->(f:File)
        RETURN f.path, type(r) as change_type, r
        ORDER BY f.path
    "#;
    
    db.query(query).await
}

// Which commands typically modify these files?
pub async fn common_file_modifiers(db: &dyn GraphDB, file_pattern: &str) -> Result<Vec<CommandPattern>> {
    let query = r#"
        MATCH (c:Command)-[:Writes|Modifies]->(f:File)  
        WHERE f.path CONTAINS $pattern
        RETURN c.cmd, COUNT(*) as frequency
        ORDER BY frequency DESC
        LIMIT 20
    "#;
    
    db.query(query).await
}
```

#### 3. Agent Intelligence Queries
```rust
// Track agent behavior patterns
pub async fn agent_command_patterns(db: &dyn GraphDB, agent_id: &str) -> Result<Vec<AgentPattern>> {
    let query = r#"
        MATCH (a:Agent {agent_id: $agent_id})-[:Executes]->(c:Command)
        RETURN c.cmd, COUNT(*) as frequency, AVG(c.duration_ms) as avg_duration
        ORDER BY frequency DESC
    "#;
    
    db.query(query).await
}
```

### Graph Backend Plugin Architecture

#### Recommended Approach: Compile-Time Extensibility

Based on research into Rust plugin architectures, the **compile-time plugin system** is optimal:

**Why Compile-Time Plugins**:
1. **Target Audience**: Developer tool where recompilation is acceptable
2. **Safety Critical**: Command tracing security can't tolerate plugin crashes  
3. **Performance**: Graph queries must be <100ms for good UX
4. **Ecosystem Standard**: How reqwest, tokio, database ORMs handle backends

#### Crate Structure (final)
```
substrate-graph/           # Main graph crate (facade)
├── src/
│   ├── lib.rs            # Public API, factory functions
│   ├── types.rs          # Shared query/result types
│   ├── error.rs          # GraphDbError enum
│   └── backends/         # Backend implementations
│       ├── mod.rs        # Backend registry
│       ├── kuzu.rs       # Kuzu implementation
│       ├── neo4j.rs      # Neo4j implementation (future)
│       └── mock.rs       # Testing backend

substrate-graph-core/      # Core traits and types
├── src/
│   ├── lib.rs            # GraphDB trait definition
│   ├── schema.rs         # Graph schema types
│   ├── query.rs          # High-level query builders
│   └── privacy.rs        # Privacy filtering
```

#### Refined GraphDB Trait
```rust
use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum GraphDbError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),
    
    #[error("Query failed: {0}")]
    QueryError(String),
    
    #[error("Schema migration failed: {0}")]
    SchemaError(String),
    
    #[error("Backend-specific error: {0}")]
    BackendError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    pub backend_type: String, // "kuzu", "neo4j", "mock"
    pub db_path: PathBuf,
    pub privacy_config: PrivacyConfig,
    pub performance_config: PerformanceConfig,
    // Backend-specific configs as serde_json::Value
    pub backend_config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub ignore_paths: Vec<String>,        // Don't index these paths
    pub hash_sensitive_files: bool,       // Hash content instead of storing
    pub max_command_length: usize,        // Truncate long commands
    pub exclude_env_vars: Vec<String>,    // Don't store these env vars
}

#[async_trait]
pub trait GraphDB: Send + Sync {
    /// Create and initialize a new database connection
    async fn connect(config: &GraphConfig) -> Result<Box<dyn GraphDB>, GraphDbError>
    where Self: Sized;
    
    /// Execute schema migrations/setup
    async fn ensure_schema(&self) -> Result<(), GraphDbError>;
    
    /// High-level trace ingestion (preferred)
    async fn ingest_trace_span(&self, span: &TraceSpan) -> Result<(), GraphDbError>;
    
    /// Batch ingestion for performance
    async fn ingest_batch(&self, spans: Vec<TraceSpan>) -> Result<(), GraphDbError>;
    
    /// High-level semantic queries
    async fn query_what_changed(&self, span_id: &str) -> Result<Vec<FileChange>, GraphDbError>;
    async fn query_command_dependencies(&self, span_id: &str) -> Result<Vec<CommandDep>, GraphDbError>;
    async fn query_security_patterns(&self, session_id: &str) -> Result<Vec<SecurityAlert>, GraphDbError>;
    
    /// Raw query interface for advanced users
    async fn raw_query(&self, query: &str) -> Result<QueryResult, GraphDbError>;
    
    /// Database maintenance
    async fn cleanup_old_data(&self, older_than: chrono::DateTime<chrono::Utc>) -> Result<(), GraphDbError>;
}
```

#### Backend Implementation Structure
```toml
# substrate-graph/Cargo.toml
[features]
default = ["backend-kuzu"]

# Built-in backends (compile-time)
backend-kuzu = ["kuzu"]           # Default, high-performance
backend-mock = []                 # Testing only
backend-neo4j = ["neo4j"]         # Future enterprise option

# Advanced features
privacy-controls = []             # Enhanced privacy filtering
performance-optimized = []        # Advanced caching and indexing

[dependencies]
substrate-graph-core = { path = "../substrate-graph-core" }
kuzu = { version = "0.11", optional = true }
neo4j = { version = "0.1", optional = true } 
tokio = { version = "1.0", features = ["rt"] }
async-trait = "0.1"
```

### Kuzu Backend Implementation Plan (agent‑executable)

#### 1. **Core KuzuBackend Implementation** (1 week)
```rust
// substrate-graph/src/backends/kuzu.rs
use kuzu::*;
use async_trait::async_trait;
use substrate_graph_core::*;

pub struct KuzuBackend {
    database: Database,
    connection: Connection,
    config: GraphConfig,
}

#[async_trait]
impl GraphDB for KuzuBackend {
    async fn connect(config: &GraphConfig) -> Result<Box<dyn GraphDB>, GraphDbError> {
        let db_path = &config.db_path;
        std::fs::create_dir_all(db_path)?;
        
        let database = Database::new(db_path.to_string_lossy(), kuzu::SystemConfig::default());
        let connection = Connection::new(&database);
        
        let backend = Box::new(KuzuBackend {
            database,
            connection, 
            config: config.clone(),
        });
        
        // Ensure schema is set up
        backend.ensure_schema().await?;
        
        Ok(backend)
    }
    
    async fn ensure_schema(&self) -> Result<(), GraphDbError> {
        // Create all node tables
        self.connection.query("
            CREATE NODE TABLE IF NOT EXISTS Command(
                span_id STRING,
                session_id STRING,
                cmd STRING,
                exit_code INT64,
                duration_ms INT64,
                component STRING,
                cwd STRING,
                timestamp TIMESTAMP,
                PRIMARY KEY (span_id)
            )
        ").await?;
        
        // Create relationship tables...
        // (Full schema implementation)
    }
    
    async fn ingest_trace_span(&self, span: &TraceSpan) -> Result<(), GraphDbError> {
        // Convert trace span to graph nodes and edges
        let filtered_span = self.apply_privacy_filters(span)?;
        
        // Insert command node
        self.connection.query("
            CREATE (:Command {
                span_id: $1, session_id: $2, cmd: $3, 
                exit_code: $4, duration_ms: $5, timestamp: $6
            })
        ").bind(&[
            &filtered_span.span_id,
            &filtered_span.session_id,
            &filtered_span.cmd,
            &filtered_span.exit_code,
            &filtered_span.duration_ms,
            &filtered_span.timestamp,
        ]).await?;
        
        // Insert file relationships
        if let Some(fs_diff) = &filtered_span.fs_diff {
            for write in &fs_diff.writes {
                self.ingest_file_relationship(&filtered_span.span_id, write, "WRITES").await?;
            }
            // Handle mods, deletes...
        }
        
        // Insert network relationships  
        for scope in &filtered_span.scopes_used {
            self.ingest_network_relationship(&filtered_span.span_id, scope).await?;
        }
        
        Ok(())
    }
}
```

#### 2. **JSONL Ingestion Pipeline** (1 week)
```rust
// substrate-graph/src/ingestion.rs
pub struct TraceIngester {
    graph_db: Box<dyn GraphDB>,
    privacy_config: PrivacyConfig,
}

impl TraceIngester {
    /// Ingest from live trace file (tail -f style)
    pub async fn ingest_live(
        &mut self, 
        trace_file: &Path
    ) -> Result<(), IngestionError> {
        let mut file = tokio::fs::File::open(trace_file).await?;
        let mut lines = BufReader::new(file).lines();
        
        while let Some(line) = lines.next_line().await? {
            if let Ok(span) = self.parse_trace_line(&line)? {
                if self.should_ingest(&span) {
                    self.graph_db.ingest_trace_span(&span).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Batch ingest from historical trace files
    pub async fn ingest_batch(&mut self, trace_file: &Path) -> Result<(), IngestionError> {
        let content = tokio::fs::read_to_string(trace_file).await?;
        let mut spans = Vec::new();
        
        for line in content.lines() {
            if let Ok(span) = self.parse_trace_line(line)? {
                if self.should_ingest(&span) {
                    spans.push(span);
                }
            }
        }
        
        self.graph_db.ingest_batch(spans).await?;
        Ok(())
    }
}
```

#### 3. **Privacy Controls** (3 days)
```rust
// substrate-graph-core/src/privacy.rs
pub struct PrivacyFilter {
    ignore_patterns: Vec<regex::Regex>,
    sensitive_commands: HashSet<String>,
    max_command_args: usize,
}

impl PrivacyFilter {
    pub fn apply_to_span(&self, span: &TraceSpan) -> Result<TraceSpan, PrivacyError> {
        let mut filtered = span.clone();
        
        // Filter sensitive command arguments
        if self.is_sensitive_command(&span.cmd) {
            filtered.cmd = self.redact_command_args(&span.cmd);
        }
        
        // Filter file paths
        if let Some(fs_diff) = &mut filtered.fs_diff {
            fs_diff.writes.retain(|p| !self.should_ignore_path(p));
            fs_diff.mods.retain(|p| !self.should_ignore_path(p));
            fs_diff.deletes.retain(|p| !self.should_ignore_path(p));
        }
        
        // Hash sensitive file contents instead of storing
        if self.should_hash_content(&filtered) {
            filtered.content_hash = Some(self.compute_content_hash(&filtered)?);
            filtered.content = None; // Remove actual content
        }
        
        Ok(filtered)
    }
}
```

### Implementation Tasks & Timeline

#### Phase 1: Core Graph Backend (2 weeks)
- [ ] **Kuzu Backend Implementation** (1 week)
  - Complete KuzuBackend struct with all GraphDB methods
  - Schema creation with proper node/edge types  
  - Connection management and error handling
  - Basic query implementation
  
- [ ] **Privacy & Filtering** (3 days)  
  - Privacy configuration system
  - Path filtering and content redaction
  - Sensitive command detection
  
- [ ] **Core Integration** (4 days)
  - Factory function for backend selection
  - Configuration loading from files/env vars
  - Error handling and graceful degradation
  - Unit tests for all components

#### Phase 2: Ingestion Pipeline (1 week)  
- [ ] **JSONL Ingestion** (4 days)
  - Live trace tailing with async file reading
  - Batch ingestion for historical data  
  - Deduplication and incremental updates
  - Performance optimization (chunking, buffering)
  
- [ ] **Schema Validation** (2 days)
  - Validate trace spans before ingestion
  - Schema migration handling
  - Data consistency checks
  
- [ ] **CLI Integration** (1 day)
  - `substrate graph ingest <file>` command
  - `substrate graph status` for health checking

#### Phase 3: Query Interface (1 week)
- [ ] **High-Level Queries** (4 days)
  - Implement semantic query functions  
  - Security analysis queries
  - Development intelligence queries
  - Agent behavior analysis
  
- [ ] **CLI Query Commands** (2 days)
  - `substrate graph what-changed <span_id>`
  - `substrate graph security-alerts [session_id]`
  - `substrate graph agent-patterns <agent_id>`
  
- [ ] **Performance Optimization** (1 day)
  - Query caching and indexing
  - Batch query optimization  
  - Memory usage optimization

### Alternative Backend Support Strategy

#### Future Backend Addition Process
When users need different graph databases:

1. **Create Backend Crate**:
   ```bash
   cargo new substrate-graph-neo4j
   # Implement GraphDB trait for Neo4j
   ```

2. **Add Feature Flag**:
   ```toml
   # Add to substrate-graph/Cargo.toml
   backend-neo4j = ["neo4j", "substrate-graph-neo4j"]
   ```

3. **Update Factory**:
   ```rust
   // Add to create_graph_db()
   #[cfg(feature = "backend-neo4j")]
   "neo4j" => Ok(Box::new(Neo4jBackend::connect(config).await?)),
   ```

4. **User Compilation**:
   ```bash
   cargo build --features backend-neo4j
   ```

#### Potential Future Backends
- **Neo4j**: Enterprise-grade with clustering
- **ArangoDB**: Multi-model database  
- **TigerGraph**: High-performance analytics
- **MemGraph**: In-memory for speed
- **DGraph**: Distributed graph database
- **Custom**: Organization-specific backends

### Configuration & Usage

#### Graph Configuration File
```yaml
# ~/.substrate/graph-config.yaml
backend: "kuzu"  # kuzu, neo4j, mock
db_path: "~/.substrate/graph"

privacy:
  ignore_paths:
    - "*/node_modules/*"
    - "*/.git/*"  
    - "*/target/*"
    - "*/.cache/*"
  hash_sensitive_files: true
  max_command_length: 500
  exclude_env_vars: ["API_KEY", "SECRET", "PASSWORD"]

performance:
  batch_size: 1000
  max_memory_mb: 512
  query_timeout_ms: 5000
  enable_indexing: true

kuzu_config:
  memory_limit: "1GB"
  thread_count: 4
  
neo4j_config:  # Future
  uri: "bolt://localhost:7687"
  username: "neo4j"
  password_env: "NEO4J_PASSWORD"
```

#### CLI Integration
```bash
# Enable graph intelligence
export SUBSTRATE_GRAPH=enabled

# Run commands with graph collection
substrate -c "npm install"
substrate -c "git clone repo.git"  
substrate -c "cargo build"

# Query the graph
substrate graph what-changed 01915c83-1234-7890-abcd-123456789012
substrate graph security-alerts
substrate graph agent-patterns claude-4

# Batch operations
substrate graph ingest ~/.substrate/trace-2024-09.jsonl  
substrate graph cleanup --older-than 30d
substrate graph export --format cypher > graph-backup.cyp
```

CLI handlers (shell)
- Implement handlers that construct a `GraphService` from config and call:
  - `graph ingest <file>` → `TraceIngester::ingest_batch(file)`
  - `graph status` → connectivity/version check and basic stats (node/edge counts)
  - `graph what-changed <span_id> [--limit N]` → call `service.what_changed(span_id, limit)` and render
  - `graph cleanup --older-than <duration>` → call `service.cleanup_old_data(cutoff)`

Build notes (Kuzu)
- Kuzu static build requires `cmake`, a C++ toolchain, and ~5–10 minutes compile time; dynamic link requires a system Kuzu.
- Provide a `--features kuzu-static` path for CI where time permits; otherwise use `mock` backend for tests.

### Success Criteria

#### Graph Intelligence Features
- [ ] Kuzu backend fully operational with schema
- [ ] Live JSONL ingestion from trace files
- [ ] Privacy filtering with configurable rules
- [ ] 10+ semantic query functions implemented  
- [ ] CLI commands for all major operations
- [ ] Performance <100ms for typical queries

#### Plugin Architecture  
- [ ] Clean compile-time backend selection
- [ ] Factory pattern for easy backend addition
- [ ] Comprehensive configuration system
- [ ] Documentation for adding new backends
- [ ] Example Neo4j backend implementation
- [ ] Migration guide from current scaffold

---

## Part C: Combined Implementation Strategy

### Integration Benefits
Implementing both features together provides synergies:
- **Shared Async Runtime**: Both use tokio, efficient resource sharing
- **Agent-Graph Integration**: Agents can query graph for intelligent analysis
- **Concurrent Graph Updates**: Graph ingestion while agents communicate
- **Unified CLI**: Consistent user experience across all advanced features

### Timeline & Effort
**Total Phase 4.5**: 4-6 weeks
- **Week 1-2**: AsyncRepl implementation and testing
- **Week 3-4**: Graph backend core implementation  
- **Week 5**: Graph ingestion pipeline and privacy controls
- **Week 6**: Integration testing, optimization, documentation

### Risk Assessment

---

# Appendix A: Agent Hub (Docking/Registry) – Minimal Implementation in 4.5

Goal: allow multiple agents (Claude/Codex/Gemini/Cline/…) to register and publish status/events; shell subscribes for live feed and status panel. Keep it lightweight in 4.5.

Crate: `agent-hub` (new) or extend `host-proxy` with a hub module.

API (Unix socket HTTP + WS):
- `POST /v1/agents/register` { agent_id, name, project?, capabilities } → 200
- `POST /v1/agents/:id/status` { state: active|idle|error, details? }
- `POST /v1/events` AgentEvent (see Part A) → broadcast
- `GET /v1/events/stream` (WebSocket): server pushes AgentEvent as JSON lines
- `GET /v1/agents` → list with last_seen/status

Hub internals
- `tokio::sync::broadcast::channel<AgentEvent>(N)` for fan‑out to shell and other consumers.
- In‑memory registry `HashMap<agent_id, AgentRecord>` with last_seen and state.

Shell
- On startup, connect to hub events stream (configurable `SUBSTRATE_AGENT_HUB_SOCK` or default path) and feed events to the non‑polling renderer (Path A).
- `substrate agents list` calls `GET /v1/agents` and renders a table.

Acceptance
- Two agents register; shell shows both in `agents list`; status changes stream to the prompt without jitter or CPU spin.

Transport Notes
- Unix: default to Unix domain socket at `~/.substrate/hub.sock` (configurable via `SUBSTRATE_AGENT_HUB_SOCK`).
- Windows: default to TCP `127.0.0.1:9876` (configurable via `SUBSTRATE_AGENT_HUB_ADDR`); shell auto-detects platform and connects accordingly.

---

# Appendix B: Path A Renderer – Code Details

Non‑polling rendering thread
```rust
// Start once on shell init
let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<AgentEvent>();
AGENT_EVENT_TX.set(tx.clone()); // global once_cell

std::thread::spawn(move || {
    // block forever without polling
    while let Some(evt) = rx.blocking_recv() {
        redraw_suspended(|out| {
            use std::io::Write;
            writeln!(out, "[{}] {}", evt.agent_id, evt.kind_string()).ok();
        });
    }
});
```

Reedline suspension helper
```rust
fn redraw_suspended<F: FnOnce(&mut std::io::Stdout)>(f: F) {
    // Acquire editor lock / suspend guard
    // Clear current prompt line: "\r\x1b[K"
    let mut out = std::io::stdout();
    write!(out, "\r\x1b[K").ok();
    f(&mut out);
    // Reprint prompt by signaling the editor (implementation detail in shell)
}
```

Test notes
- Verify idle CPU with `top`/`htop` while agents are connected and idle.
- Emit 1k events/sec from a test agent; renderer should keep up without prompt corruption.

---

# Appendix C: Graph Implementation – Final Agent Instructions

Consolidation
- Ensure trace crate no longer links to Kuzu; substrate‑graph provides Kuzu backend under `kuzu-static` or `kuzu-dylib` features.

Schema (parameterized; avoid string injection)
```sql
CREATE NODE TABLE IF NOT EXISTS Command(
  span_id STRING PRIMARY KEY,
  session_id STRING, cmd STRING, exit_code INT64,
  duration_ms INT64, component STRING, cwd STRING, timestamp TIMESTAMP
);
CREATE NODE TABLE IF NOT EXISTS File(path STRING PRIMARY KEY, type STRING, permissions STRING, size INT64);
CREATE REL TABLE IF NOT EXISTS Writes(FROM Command TO File, bytes_written INT64, content_hash STRING);
-- (others as in main text)
```

Backend
```rust
// kuzu backend execute with parameters (pseudo-code)
conn.query_with_params("CREATE (:Command { span_id: $1, cmd: $2, exit_code: $3, timestamp: $4 })",
  &[&span.span_id, &span.cmd, &span.exit.unwrap_or(-1), &span.ts])?;
```

Ingestion
- `TraceIngester::ingest_live(path)` tails JSONL; for each line parse Span and FsDiff; apply privacy filters; call `ingest_span`.

Queries
- Implement `what_changed(span_id)` to traverse Writes|Modifies|Deletes rels; add `LIMIT` param.

Acceptance
- Ingest a real trace.jsonl; `what-changed` matches shell’s printed FsDiff; typical queries return in <100ms.

---

# Appendix D: Phase 4.5 Checklists

Concurrent Output (Path A)
- [ ] Agent Hub minimal API + broadcast
- [ ] Shell renderer thread; idle CPU verified; no prompt corruption
- [ ] `substrate agents list` command

Graph Intelligence
- [ ] Kuzu backend behind features; connect/ensure_schema implemented
- [ ] Trace ingestion (live + batch) with privacy filters
- [ ] High‑level queries (what‑changed, security‑alerts minimal)
- [ ] CLI: ingest, status, what‑changed

Perf & Tests
- [ ] Path A idle CPU < 0.1%, 1k events/sec keeps up
- [ ] Graph query p50 < 100ms on typical graph size
**Medium Risk**: Both are complex features with async coordination
**Mitigation**: Implement separately first, then integrate
**Fallback**: Each can be shipped independently if needed

---

**Document Created**: September 3, 2025  
**Prerequisites**: Phase 4 Core Complete ✅  
**Status**: Comprehensive plan ready for implementation  
**Estimated Completion**: Phase 4.5 delivers advanced features for enterprise-ready platform
