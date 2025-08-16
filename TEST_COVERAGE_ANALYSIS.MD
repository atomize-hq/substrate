# Test Coverage Improvement Plan

## Current Coverage Status

As of the latest `cargo tarpaulin` analysis (excluding third_party/reedline):

### Overall Statistics

- **Total Coverage**: ~50% (estimated for our crates)
- **Critical Gaps**: 2 modules with 0% coverage
- **Target Goal**: 70% overall coverage

### Coverage by Crate

| Crate          | Module              | Current Coverage | Lines Covered | Priority     |
| -------------- | ------------------- | ---------------- | ------------- | ------------ |
| **shell**      | lib.rs              | 50%              | 401/807       | Medium       |
| **shell**      | **pty_exec.rs**     | **0%**           | **0/237**     | **CRITICAL** |
| **shell**      | **host_decider.rs** | **0%**           | **0/14**      | **HIGH**     |
| **shell**      | main.rs             | 75%              | 3/4           | Low          |
| **shim**       | context.rs          | 69%              | 40/58         | Low          |
| **shim**       | **exec.rs**         | **19%**          | **25/129**    | **HIGH**     |
| **shim**       | logger.rs           | 88%              | 99/112        | Complete     |
| **shim**       | resolver.rs         | 96%              | 22/23         | Complete     |
| **supervisor** | **lib.rs**          | **23%**          | **16/70**     | **Medium**   |
| **supervisor** | **main.rs**         | **0%**           | **0/22**      | **Low**      |
| **common**     | lib.rs              | 94%              | 17/18         | Complete     |

## Priority 1: Critical Gaps (Target: 2 weeks)

### 1.1 PTY Execution Module (pty_exec.rs) - 0% Coverage

**Target Coverage: 60%** (142 lines)

#### Why Critical

- Core functionality that was recently debugged
- Handles all terminal emulation
- Any regression here breaks interactive commands
- Most complex untested code (signals, threading, platform-specific)

#### Enhanced Test Implementation with OsApi Trait Pattern

First, refactor to isolate system calls for better testability:

```rust
// crates/shell/src/pty/os_api.rs - NEW FILE
pub trait OsApi: Send + Sync {
    fn fork(&self) -> Result<libc::pid_t, String>;
    fn openpty(&self, size: PtySize) -> Result<PtyPair, String>;
    fn ioctl_winsize(&self, fd: RawFd, size: &Winsize) -> Result<(), String>;
    fn kill(&self, pid: libc::pid_t, signal: libc::c_int) -> Result<(), String>;
}

pub struct LiveOsApi;

impl OsApi for LiveOsApi {
    fn fork(&self) -> Result<libc::pid_t, String> {
        let result = unsafe { libc::fork() };
        if result < 0 {
            Err(format!("fork failed: {}", std::io::Error::last_os_error()))
        } else {
            Ok(result)
        }
    }

    fn openpty(&self, size: PtySize) -> Result<PtyPair, String> {
        portable_pty::native_pty_system()
            .openpty(size)
            .map_err(|e| e.to_string())
    }

    // ... other implementations
}

#[cfg(test)]
pub struct MockOsApi {
    pub fork_result: Result<libc::pid_t, String>,
    pub openpty_result: Result<MockPtyPair, String>,
    pub ioctl_calls: Arc<Mutex<Vec<Winsize>>>,
}
```

Now update PTY implementation to use the trait:

```rust
// crates/shell/src/pty_exec.rs - REFACTORED
pub fn execute_with_pty_impl<O: OsApi>(
    cmd: &str,
    args: &[String],
    os_api: Arc<O>,
) -> Result<PtyExitStatus> {
    let pty_pair = os_api.openpty(get_terminal_size()?)?;
    let pid = os_api.fork()?;

    if pid == 0 {
        // Child process
        setup_child_process(&pty_pair)?;
        exec_command(cmd, args)?;
    } else {
        // Parent process
        handle_pty_io(&pty_pair, pid, os_api.clone())?;
    }

    Ok(PtyExitStatus::new(0))
}

// Production entry point
pub fn execute_with_pty(cmd: &str, args: &[String]) -> Result<PtyExitStatus> {
    execute_with_pty_impl(cmd, args, Arc::new(LiveOsApi))
}
```

Comprehensive test suite:

```rust
// crates/shell/src/pty_exec.rs - Test module
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_pty_fork_failure() {
        let mock_api = Arc::new(MockOsApi {
            fork_result: Err("fork failed".into()),
            ..Default::default()
        });

        let result = execute_with_pty_impl("ls", &[], mock_api);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fork failed"));
    }

    #[test]
    fn test_pty_openpty_failure() {
        let mock_api = Arc::new(MockOsApi {
            openpty_result: Err("no ptys available".into()),
            ..Default::default()
        });

        let result = execute_with_pty_impl("ls", &[], mock_api);
        assert!(result.is_err());
    }

    #[test]
    fn test_pty_resize_tracking() {
        let ioctl_calls = Arc::new(Mutex::new(Vec::new()));
        let mock_api = Arc::new(MockOsApi {
            fork_result: Ok(12345),
            ioctl_calls: ioctl_calls.clone(),
            ..Default::default()
        });

        // Simulate SIGWINCH
        send_resize_signal(&mock_api, 80, 24);

        let calls = ioctl_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].ws_col, 80);
        assert_eq!(calls[0].ws_row, 24);
    }

    #[test]
    fn test_signal_handling() {
        let kill_count = Arc::new(AtomicU32::new(0));
        let mock_api = Arc::new(MockOsApi {
            kill_result: Ok(()),
            kill_count: kill_count.clone(),
            ..Default::default()
        });

        // Test SIGTERM forwarding
        forward_signal(&mock_api, 12345, libc::SIGTERM);
        assert_eq!(kill_count.load(Ordering::SeqCst), 1);
    }
}
```

#### Specific Functions to Test

1. `execute_with_pty()` - Main entry point (mock required)
2. `get_terminal_size()` - Platform-specific size detection
3. `handle_pty_io()` - I/O multiplexing (mock required)
4. `MinimalTerminalGuard` - Terminal state management
5. `CurrentPtyGuard` - Global PTY tracking
6. Signal handling (SIGWINCH) - Unix only

### 1.2 Host Decider Module (host_decider.rs) - 0% Coverage

**Target Coverage: 80%** (11 lines)

#### Test Implementation Plan

```rust
// crates/shell/src/host_decider.rs - Add tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_decider_detects_pty_commands() {
        let decider = SubstrateHostDecider;

        // Should trigger PTY
        assert!(decider.should_execute_host_command("vim"));
        assert!(decider.should_execute_host_command("ssh host"));
        assert!(decider.should_execute_host_command("docker run -it ubuntu"));

        // Should not trigger PTY
        assert!(!decider.should_execute_host_command("ls"));
        assert!(!decider.should_execute_host_command("echo hello"));
    }

    #[test]
    fn test_parse_command_line() {
        let decider = SubstrateHostDecider;

        // Test command parsing logic
        assert!(decider.should_execute_host_command("python"));
        assert!(!decider.should_execute_host_command("python script.py"));
    }
}
```

## Priority 2: Low Coverage Modules (Target: 3 weeks)

### 2.1 Shim Execution Module (exec.rs) - 19% Coverage

**Target Coverage: 50%** (65 lines)

#### Missing Test Areas

- Command execution with various exit codes
- Signal handling during execution
- Environment variable propagation
- Working directory changes
- Error cases (command not found, permission denied)

#### Enhanced Testing with Property-Based Tests

```rust
// Standard unit tests
#[test]
fn test_execute_command_success() {
    let result = execute_with_args(&["echo", "hello"]);
    assert!(result.is_ok());
}

#[test]
fn test_execute_command_not_found() {
    let result = execute_with_args(&["nonexistent_command_xyz"]);
    assert!(result.is_err());
}

// Property-based testing for command parsing
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_command_never_panics(input in "\\PC*") {
        // Any printable character sequence shouldn't crash parser
        let _ = parse_command_line(&input);
    }

    #[test]
    fn test_quote_handling_preserves_content(
        content in "[a-zA-Z0-9 ]+",
        quote_type in prop::sample::select(vec!["'", "\""])
    ) {
        let quoted = format!("{}{}{}", quote_type, content, quote_type);
        let parsed = parse_command_line(&quoted);
        prop_assert_eq!(parsed.args.len(), 1);
        prop_assert_eq!(parsed.args[0], content);
    }

    #[test]
    fn test_escape_sequence_handling(
        prefix in "[a-z]+",
        escape in prop::sample::select(vec!["\\n", "\\t", "\\\\", "\\\""])
    ) {
        let input = format!("{}{}rest", prefix, escape);
        let parsed = parse_command_line(&input);
        // Should handle escape sequences correctly
        prop_assert!(parsed.is_ok());
    }

    #[test]
    fn test_pipe_parsing_consistency(commands in prop::collection::vec("[a-z]+", 1..5)) {
        let piped = commands.join(" | ");
        let parsed = parse_pipeline(&piped);
        prop_assert_eq!(parsed.len(), commands.len());
    }

    #[test]
    fn test_environment_variable_expansion(
        var_name in "[A-Z_][A-Z0-9_]*",
        var_value in "[a-zA-Z0-9]+",
    ) {
        std::env::set_var(&var_name, &var_value);
        let input = format!("echo ${}", var_name);
        let parsed = parse_and_expand(&input);
        prop_assert!(parsed.args.contains(&var_value));
        std::env::remove_var(&var_name);
    }
}
```

### 2.2 Supervisor Module - 23% Coverage

**Target Coverage: 50%** (35 lines)

#### Missing Test Areas

- Process spawning with shimmed PATH
- Environment setup
- Signal forwarding
- Cleanup on exit

## Priority 3: Improve Existing Coverage (Target: 4 weeks)

### 3.1 Documentation Tests for Public APIs

**Quick Win: ~5% coverage boost**

Add doctests to all public functions. These serve as both documentation and tests:

````rust
// crates/shell/src/lib.rs

/// Parses a shell command line into individual arguments.
///
/// Handles quotes, escapes, and environment variable expansion.
///
/// # Examples
///
/// ```
/// use substrate::parse_command;
///
/// let args = parse_command("ls -la 'my file.txt'");
/// assert_eq!(args, vec!["ls", "-la", "my file.txt"]);
/// ```
///
/// # Complex Examples
///
/// ```
/// use substrate::parse_command;
///
/// // Handle environment variables
/// std::env::set_var("USER", "alice");
/// let args = parse_command("echo Hello $USER");
/// assert_eq!(args, vec!["echo", "Hello", "alice"]);
///
/// // Handle escape sequences
/// let args = parse_command(r#"echo "line1\nline2""#);
/// assert_eq!(args, vec!["echo", "line1\nline2"]);
/// ```
///
/// # Errors
///
/// ```
/// use substrate::parse_command;
///
/// // Unmatched quotes return error
/// let result = parse_command("echo 'unclosed");
/// assert!(result.is_err());
/// ```
pub fn parse_command(input: &str) -> Result<Vec<String>, ParseError> {
    // Implementation
}

/// Executes a command with PTY allocation if needed.
///
/// Automatically detects interactive commands and allocates a PTY.
///
/// # Examples
///
/// ```no_run
/// use substrate::execute_command;
///
/// // Simple command - no PTY needed
/// let status = execute_command("ls", &["-la"])?;
/// assert!(status.success());
/// ```
///
/// ```no_run
/// use substrate::execute_command;
///
/// // Interactive command - PTY allocated automatically
/// let status = execute_command("vim", &["file.txt"])?;
/// // User interacts with vim...
/// ```
pub fn execute_command(cmd: &str, args: &[&str]) -> Result<ExitStatus> {
    // Implementation
}

/// Creates a new shell configuration with default settings.
///
/// # Examples
///
/// ```
/// use substrate::ShellConfig;
///
/// let config = ShellConfig::default();
/// assert_eq!(config.history_size, 1000);
/// assert_eq!(config.prompt, "$ ");
/// ```
///
/// ```
/// use substrate::ShellConfig;
///
/// let config = ShellConfig::builder()
///     .history_size(5000)
///     .prompt(">>> ")
///     .enable_colors(true)
///     .build();
/// assert_eq!(config.history_size, 5000);
/// ```
pub struct ShellConfig {
    // Fields
}
````

### 3.2 Shell lib.rs - 50% Coverage

**Target Coverage: 70%** (565 lines)

#### Missing Test Areas

- Error handling paths
- Edge cases in command parsing
- Built-in command error cases
- Complex pipeline handling
- Signal handling during command execution

### 3.2 Real PTY Integration Tests with expectrl

Create true end-to-end tests that exercise actual PTY behavior:

```rust
// tests/pty_integration.rs
use expectrl::{spawn, ControlCode, Error, Expect};
use std::time::Duration;

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_real_pty_echo_command() {
    let mut session = spawn("substrate").expect("Failed to spawn substrate");

    // Send command and verify echo
    session.send_line("echo hello world").unwrap();
    session.expect("hello world").unwrap();

    // Verify prompt returns
    session.expect("$").unwrap();
}

#[test]
#[ignore]
fn test_real_pty_interactive_vim() {
    let mut session = spawn("substrate").expect("Failed to spawn substrate");

    // Launch vim
    session.send_line("vim test.txt").unwrap();

    // Wait for vim to start (look for tildes)
    session.expect("~").unwrap();

    // Enter insert mode
    session.send("i").unwrap();

    // Type some text
    session.send("Hello from test").unwrap();

    // Exit insert mode
    session.send(&[0x1b]).unwrap(); // ESC

    // Save and quit
    session.send(":wq\r").unwrap();

    // Verify return to shell
    session.expect("$").unwrap();

    // Verify file was created
    session.send_line("cat test.txt").unwrap();
    session.expect("Hello from test").unwrap();
}

#[test]
#[ignore]
fn test_real_pty_resize_handling() {
    use nix::pty::Winsize;

    let mut session = spawn("substrate").expect("Failed to spawn substrate");

    // Get initial size
    let initial_size = session.get_winsize().unwrap();

    // Start a command that displays size
    session.send_line("tput cols; tput lines").unwrap();
    session.expect(&initial_size.ws_col.to_string()).unwrap();
    session.expect(&initial_size.ws_row.to_string()).unwrap();

    // Resize the PTY
    let new_size = Winsize {
        ws_row: 40,
        ws_col: 100,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    session.set_winsize(new_size).unwrap();

    // Verify new size is reflected
    session.send_line("tput cols; tput lines").unwrap();
    session.expect("100").unwrap();
    session.expect("40").unwrap();
}

#[test]
#[ignore]
fn test_real_pty_signal_forwarding() {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

    let mut session = spawn("substrate").expect("Failed to spawn substrate");

    // Start a long-running command
    session.send_line("sleep 30").unwrap();

    // Send SIGINT (Ctrl+C)
    session.send(ControlCode::ETX).unwrap(); // Ctrl+C

    // Verify command was interrupted
    session.expect("^C").unwrap();
    session.expect("$").unwrap(); // Back at prompt
}

#[test]
#[ignore]
fn test_real_pty_escape_sequences() {
    let mut session = spawn("substrate").expect("Failed to spawn substrate");

    // Test color escape sequences
    session.send_line("echo -e '\\033[31mRed Text\\033[0m'").unwrap();

    // The raw output should contain the escape sequences
    let output = session.expect(regex::Regex::new(r"\x1b\[31m").unwrap()).unwrap();
    assert!(output.matches().len() > 0);
}

// Platform-specific test
#[test]
#[ignore]
#[cfg(unix)]
fn test_real_pty_job_control() {
    let mut session = spawn("substrate").expect("Failed to spawn substrate");

    // Start a background job
    session.send_line("sleep 30 &").unwrap();
    session.expect("[1]").unwrap(); // Job number

    // List jobs
    session.send_line("jobs").unwrap();
    session.expect("Running").unwrap();

    // Bring to foreground
    session.send_line("fg").unwrap();

    // Kill it
    session.send(ControlCode::ETX).unwrap(); // Ctrl+C
    session.expect("$").unwrap();
}
```

## Implementation Strategy

### Phase 1: Critical Tests (Week 1-2)

1. **Day 1-3**: Implement pty_exec.rs basic tests

   - Terminal size detection
   - PTY allocation
   - Exit status handling

2. **Day 4-5**: Implement host_decider.rs tests

   - Command detection logic
   - Parsing edge cases

3. **Day 6-10**: Create PTY mock framework

   - Mock PTY implementation
   - Test harness for PTY operations

4. **Day 11-14**: Integration tests for PTY
   - Full execution flow
   - Signal handling
   - Error recovery

### Phase 2: Coverage Expansion (Week 3-4)

1. **Week 3**: Shim execution tests

   - Command execution
   - Error handling
   - Environment management

2. **Week 4**: Supervisor tests
   - Process management
   - PATH manipulation
   - Cleanup testing

### Phase 3: Comprehensive Coverage (Week 5-6)

1. **Week 5**: Shell lib.rs expansion

   - Error paths
   - Edge cases
   - Complex scenarios

2. **Week 6**: Integration test suite
   - End-to-end workflows
   - Performance tests
   - Stress tests

## Enhanced Testing Infrastructure

### 1. Mock PTY Implementation

```rust
trait PtyMock {
    fn allocate(&self) -> Result<MockPty>;
    fn write(&mut self, data: &[u8]) -> Result<usize>;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}
```

### 2. Property-Based Testing with proptest

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_pty_buffer_handling(data: Vec<u8>) {
        let mut pty = MockPty::new();
        pty.write(&data)?;
        let mut buf = vec![0u8; data.len()];
        let read = pty.read(&mut buf)?;
        prop_assert_eq!(read, data.len());
    }
}
```

### 3. Fuzzing Infrastructure

```bash
# Initialize fuzzing
cargo fuzz init

# Add PTY fuzzing target
cat > fuzz/fuzz_targets/pty_escape.rs << 'EOF'
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = parse_escape_sequence(data);
});
EOF

# Run fuzzing (3 minutes per target)
cargo fuzz run pty_escape -- -max_total_time=180
```

### 4. Mutation Testing Setup

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation testing on critical modules
cargo mutants --file crates/shell/src/pty_exec.rs

# Check mutation score
cargo mutants --output json | jq '.score'
```

### 5. Performance Benchmarking

```rust
// benches/pty_performance.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn pty_benchmark(c: &mut Criterion) {
    c.bench_function("pty_write_1mb", |b| {
        let mut pty = setup_pty();
        let data = vec![b'x'; 1_000_000];
        b.iter(|| pty.write_all(&data))
    });
}

criterion_group!(benches, pty_benchmark);
criterion_main!(benches);
```

### 6. Test Fixtures Organization

```
tests/
├── fixtures/
│   ├── pty_sequences/
│   │   ├── valid_escapes.bin
│   │   ├── malformed.bin
│   │   └── stress_test.bin
│   ├── commands/
│   │   ├── interactive.txt
│   │   └── batch.txt
│   └── golden/
│       └── expected_outputs.json
```

### 7. CI/CD Pipeline Integration

```yaml
name: Comprehensive Testing
on: [push, pull_request]

jobs:
  test-matrix:
    strategy:
      matrix:
        test-type: [unit, integration, property, fuzz, mutation, benchmark]

    steps:
      - name: Run ${{ matrix.test-type }} tests
        run: |
          case "${{ matrix.test-type }}" in
            unit)
              cargo test --lib
              ;;
            integration)
              cargo test --test '*'
              ;;
            property)
              cargo test --features proptest
              ;;
            fuzz)
              cargo fuzz run pty_escape -- -max_total_time=60
              ;;
            mutation)
              cargo mutants --timeout 30 --jobs 4
              ;;
            benchmark)
              cargo bench --bench performance
              ;;
          esac
```

## Success Metrics

### Tiered Coverage Goals

Based on comprehensive analysis of codebase complexity (5,598 lines total, ~50% current coverage):

#### Confidence Levels

- **65% Coverage**: Minimum Acceptable (High Confidence)
  - Achievable with identified 417 lines + basic integration tests
  - Represents solid improvement from 50% baseline
  - Industry standard for terminal emulators (40-60%)
- **70% Coverage**: Target Goal (Medium Confidence)
  - Requires 1,120 additional lines covered
  - Achievable with full plan implementation
  - Upper quartile for system-level Rust projects
- **75% Coverage**: Stretch Goal (Low Confidence)
  - Only if no blocking issues arise
  - Requires extensive mocking infrastructure
  - Would place project in top tier

### Module-Specific Targets

- **PTY Execution (pty_exec.rs)**: 50% → 60% → 70% (currently 0%)
- **Critical Modules**: 50% → 60% → 70%
- **Public APIs**: 70% → 80% → 90%

### Quality Metrics

- **Mutation Score**: ≥60% for critical modules
- **Property Test Coverage**: All state machines and parsers
- **Fuzz Test Duration**: 3 hours cumulative per week
- **Performance Regression Threshold**: <5% degradation
- **Test Execution Time**: <2 minutes for unit tests

### Test Type Distribution

| Test Type         | Target Count    | Priority |
| ----------------- | --------------- | -------- |
| Unit Tests        | 200+            | Critical |
| Integration Tests | 50+             | High     |
| Property Tests    | 20+             | High     |
| Fuzz Targets      | 5+              | Medium   |
| Benchmarks        | 10+             | Medium   |
| Doc Tests         | All public APIs | High     |

## Automation

### Comprehensive Test Suite Script

```bash
#!/bin/bash
# run_all_tests.sh

echo "🧪 Running Comprehensive Test Suite"

# 1. Unit & Integration Tests
echo "📦 Running unit tests..."
cargo test --lib || exit 1
cargo test --test '*' || exit 1

# 2. Coverage Check
echo "📊 Checking coverage..."
COVERAGE=$(cargo tarpaulin --print-summary | grep "Coverage" | awk '{print $1}' | sed 's/%//')
if (( $(echo "$COVERAGE < 60" | bc -l) )); then
    echo "❌ Coverage too low: $COVERAGE%"
    exit 1
fi

# 3. Property Tests
echo "🎲 Running property tests..."
cargo test --features proptest || exit 1

# 4. Mutation Testing (quick check)
echo "🧬 Running mutation tests..."
cargo mutants --timeout 10 --file crates/shell/src/pty_exec.rs || true

# 5. Performance Check
echo "⚡ Running benchmarks..."
cargo bench --bench performance -- --quick || exit 1

echo "✅ All tests passed!"
```

### Enhanced Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Quick tests only
cargo test --lib --release || exit 1
cargo fmt -- --check || exit 1
cargo clippy -- -D warnings || exit 1

# Quick coverage check on changed files
CHANGED_RS=$(git diff --cached --name-only | grep '\.rs$')
if [ ! -z "$CHANGED_RS" ]; then
    cargo tarpaulin --fail-under 60 --timeout 120 || exit 1
fi
```

### GitHub Actions Matrix

```yaml
name: Complete Test Matrix
on: [push, pull_request]

jobs:
  quick-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Quick validation
        run: |
          cargo fmt -- --check
          cargo clippy -- -D warnings
          cargo test --lib

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: cargo tarpaulin --out Xml --all-features --timeout 300
      - name: Upload to codecov
        uses: codecov/codecov-action@v3

  advanced-tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v3
      - name: Mutation testing
        run: |
          cargo install cargo-mutants
          cargo mutants --timeout 30 --jobs 2
      - name: Fuzz testing
        run: |
          cargo install cargo-fuzz
          cargo fuzz run pty_escape -- -max_total_time=60
```

## Timeline with Realistic Expectations

### Risk-Adjusted Schedule

| Week      | Focus                  | Coverage Target | Confidence | Deliverables                            |
| --------- | ---------------------- | --------------- | ---------- | --------------------------------------- |
| 1         | Infrastructure Setup   | +2%             | High       | OsApi trait, mock framework, CI setup   |
| 2         | PTY Unit Tests         | +8%             | High       | Mock-based PTY tests, signal handling   |
| 3         | PTY Integration        | +5%             | Medium     | expectrl tests, platform-specific tests |
| 4         | Parser & Shim          | +7%             | High       | Property tests, command execution       |
| 5         | Documentation & Polish | +5%             | High       | Doctests, supervisor tests              |
| 6         | Buffer & Catch-up      | +3-8%           | Low        | Address blockers, stretch goals         |
| **Total** | **6 weeks**            | **65-70%**      | **Medium** | **50% → 65-70% coverage**               |

### Key Milestones

#### Week 2 Checkpoint

- **Goal**: 58% coverage (High confidence)
- **Blocker Risk**: OsApi refactoring complexity
- **Mitigation**: Fall back to simpler mocking if needed

#### Week 4 Checkpoint

- **Goal**: 65% coverage (Medium confidence)
- **Decision Point**: Assess if 70% is achievable
- **Alternative**: Accept 65% as success if PTY tests prove too complex

#### Week 6 Final

- **Minimum**: 65% coverage (Acceptable)
- **Target**: 70% coverage (Good)
- **Stretch**: 75% coverage (Excellent)

### Risk Factors & Mitigations

| Risk                   | Impact | Probability | Mitigation                            |
| ---------------------- | ------ | ----------- | ------------------------------------- |
| PTY mocking complexity | High   | Medium      | Start with OsApi pattern early        |
| Platform differences   | Medium | High        | Use conditional compilation liberally |
| CI flakiness           | Low    | High        | Mark flaky tests as `#[ignore]`       |
| Time overrun           | Medium | Medium      | Built-in buffer week                  |
| Diminishing returns    | Low    | High        | Focus on high-value tests first       |

## Next Steps

1. **Immediate**: Start with pty_exec.rs tests (highest risk)
2. **This Week**: Complete Priority 1 items
3. **This Month**: Achieve 60% minimum coverage
4. **This Quarter**: Reach 70% target coverage

## Resources Required

- Developer time: 6 weeks
- CI infrastructure updates
- Mock framework development
- Test fixture creation

## Risk Mitigation

1. **PTY tests failing in CI**: Use conditional compilation and mocks
2. **Platform differences**: Separate Unix/Windows test suites
3. **Test maintenance burden**: Focus on high-value tests first
4. **Performance impact**: Use cargo-nextest for parallel execution
