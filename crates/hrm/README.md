# HRM (Hot Reload Module)

The Hot Reload Module provides deterministic trace replay and regression testing capabilities for Substrate. It enables replaying recorded command sequences from trace.jsonl files to verify behavior consistency and detect regressions.

## Features

- **Trace Replay**: Re-execute commands from recorded traces with environment reconstruction
- **Regression Detection**: Compare replay outputs against original executions
- **Batch Testing**: Replay multiple spans and generate comprehensive reports
- **Flexible Filtering**: Select specific spans based on commands, exit codes, or components
- **Output Comparison**: Intelligent diffing that handles non-deterministic elements (timestamps, PIDs)
- **Future World Integration**: Designed to integrate with world isolation backends when available

## Architecture

The HRM module consists of four main components:

### 1. Trace Reader (`state.rs`)
- Loads spans from trace.jsonl files
- Reconstructs execution state from replay_context
- Filters spans based on configurable criteria

### 2. Replay Engine (`replay.rs`)
- Executes commands with reconstructed environment
- Supports both direct and world-isolated execution (future)
- Handles stdin, timeout, and environment variables

### 3. Comparison Engine (`compare.rs`)
- Compares replay results with original traces
- Handles non-deterministic elements gracefully
- Categorizes divergences by severity

### 4. Regression Analysis (`regression.rs`)
- Aggregates results from batch replays
- Generates detailed regression reports
- Provides actionable recommendations

## Usage

### Basic Replay

```rust
use substrate_hrm::{replay_span, ReplayConfig};

let config = ReplayConfig {
    trace_file: PathBuf::from("~/.substrate/trace.jsonl"),
    strict: false,  // Allow minor divergences
    timeout: 300,   // 5 minute timeout
    ..Default::default()
};

let result = replay_span("span_12345", &config).await?;
if result.matched {
    println!("Replay matched original execution!");
} else {
    println!("Divergence detected: {:?}", result.divergence);
}
```

### Batch Testing

```rust
use substrate_hrm::{find_spans_to_replay, replay_batch, SpanFilter};

// Find all npm commands
let filter = SpanFilter {
    command_patterns: vec!["npm".to_string()],
    ..Default::default()
};

let spans = find_spans_to_replay(&trace_file, filter).await?;
let report = replay_batch(&spans, &config).await?;

println!("Pass rate: {:.1}%", report.pass_rate);
```

### Regression Testing

```rust
// Replay golden test traces
let golden_spans = vec!["test-1", "test-2", "test-3"];
let report = replay_batch(&golden_spans, &config).await?;

if report.critical_failures.len() > 0 {
    panic!("Critical regression detected!");
}
```

## Configuration

The `ReplayConfig` struct provides fine-grained control:

- `strict`: Fail on any divergence vs allowing minor differences
- `fresh_world`: Use isolated world for each replay (future feature)
- `ignore_timing`: Ignore timing differences in comparison
- `max_output_compare`: Limit output comparison size for performance
- `env_overrides`: Override specific environment variables

## Integration Status

### Current (PR#12 Phase 1)
- ✅ Direct execution replay
- ✅ Trace parsing and state reconstruction
- ✅ Output comparison with non-deterministic handling
- ✅ Batch replay and regression reporting
- ✅ Integration tests

### Future (When World API Stabilizes)
- 🔄 Full world isolation for replay
- 🔄 Filesystem diff verification
- 🔄 Network scope tracking
- 🔄 Deterministic replay guarantees

## Testing

Run unit tests:
```bash
cargo test -p substrate-hrm
```

Run integration tests:
```bash
cargo test -p substrate-hrm --test integration
```

## Performance

- Trace parsing: < 10ms per span
- State reconstruction: < 5ms
- Command replay: Depends on command (typically < 100ms overhead)
- Comparison: < 1ms for most outputs

## Design Decisions

1. **Phased Integration**: HRM can operate without world backends, using direct execution as a fallback. This allows immediate functionality while world APIs stabilize.

2. **Non-Deterministic Handling**: The comparison engine intelligently handles timestamps, PIDs, and other non-deterministic elements to reduce false positives.

3. **Severity Levels**: Divergences are categorized (Critical/High/Medium/Low) to help prioritize investigation efforts.

4. **Extensible Filtering**: The SpanFilter design allows complex queries while remaining simple for basic use cases.

## Future Enhancements

- Graph database integration for relationship tracking
- Machine learning for anomaly detection
- Parallel replay execution
- Visual diff reporting
- CI/CD integration helpers