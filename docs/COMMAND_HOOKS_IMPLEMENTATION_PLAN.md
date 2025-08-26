# Substrate Command Hooks Implementation Plan

## Executive Summary

This document outlines the implementation of a comprehensive command hook system for Substrate that enables fine-grained control over command execution at the shim level. The system will allow users to define hooks that can intercept, modify, block, or augment any command execution based on configurable rules, providing unprecedented control over AI agent behavior and general command execution.

## 🎯 Core Vision

Transform Substrate from a passive command tracer into an **active command mediator** that can:
- **Intercept** commands before execution for evaluation
- **Block** dangerous or unwanted commands
- **Modify** command arguments or redirect execution
- **Augment** commands with additional context or safety measures
- **Notify** external systems about command execution patterns
- **Enforce** security policies and compliance requirements

## 🏗️ Architecture Overview

### Hook Evaluation Pipeline

```
Command Invocation
        ↓
   Shim Intercept
        ↓
┌─────────────────┐
│  Hook Matcher   │ ← Pattern matching & context evaluation
└────────┬────────┘
         ↓
┌─────────────────┐
│ Hook Evaluator  │ ← Rule evaluation & action determination
└────────┬────────┘
         ↓
┌─────────────────┐
│ Action Handler  │ ← Execute hook actions
└────────┬────────┘
         ↓
   [Allow/Block/Modify]
         ↓
  Execute/Reject Command
```

### Integration Points

The hook system will integrate at **two critical points** in the existing codebase:

1. **Shim Level** (`crates/shim/src/exec.rs::run_shim()`)
   - Primary interception point before binary resolution
   - Access to full command context and environment
   - Can prevent execution entirely

2. **Shell Level** (`crates/shell/src/lib.rs::execute_command()`)
   - Secondary hook point for shell-specific commands
   - Access to shell session context
   - Can modify shell behavior

## 📋 Implementation Phases

### Phase 1: Core Hook Engine (Week 1-2)

#### 1.1 Hook Definition Schema
Create a new crate `substrate-hooks` with the following structure:

```rust
// crates/hooks/src/lib.rs
pub struct Hook {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub priority: i32,  // Higher priority hooks execute first
    pub matcher: Matcher,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}

pub enum Matcher {
    Exact(String),                    // Exact command match
    Pattern(String),                  // Regex pattern
    Glob(String),                     // Shell glob pattern
    Binary(String),                   // Specific binary name
    Composite(Vec<Matcher>),          // Multiple matchers (AND/OR)
}

pub enum Condition {
    Depth(DepthCondition),            // Based on SHIM_DEPTH
    Parent(String),                   // Parent command match
    SessionAge(Duration),             // Time since session start
    UserGroup(String),                // Unix user group
    EnvironmentVar(String, String),   // Env var check
    TimeWindow(TimeRange),            // Time of day restrictions
    RateLimit(RateLimitConfig),       // Command frequency limits
}

pub enum Action {
    Allow,                            // Continue execution
    Block(BlockConfig),               // Prevent execution
    Modify(ModifyConfig),             // Change command/args
    Notify(NotifyConfig),             // Send notifications
    Log(LogConfig),                   // Enhanced logging
    Prompt(PromptConfig),             // User confirmation
    Redirect(RedirectConfig),         // Redirect to different command
    Sandbox(SandboxConfig),           // Execute in restricted environment
    Script(ScriptConfig),             // Execute custom script/program
}
```

#### 1.2 Hook Storage & Configuration

```yaml
# ~/.substrate/hooks.yaml
version: "1.0"
hooks:
  - id: block-rm-rf
    name: "Prevent recursive deletion"
    enabled: true
    priority: 100
    matcher:
      pattern: "^rm.*(-rf|--recursive.*--force)"
    actions:
      - block:
          message: "Recursive force deletion blocked by policy"
          exit_code: 1
      - notify:
          webhook: "https://alerts.company.com/dangerous-command"
          
  - id: git-push-protection
    name: "Confirm force pushes"
    matcher:
      pattern: "^git push.*--force"
    conditions:
      - parent: "!git"  # Not already in a git command
    actions:
      - prompt:
          message: "AI agent wants to force push. Allow?"
          timeout: 30s
          default: deny
          
  - id: rate-limit-curl
    name: "Rate limit HTTP requests"
    matcher:
      exact: "curl"
    conditions:
      - rate_limit:
          max_count: 100
          window: 1m
          per: "session"
    actions:
      - log:
          level: "warning"
          message: "Rate limit approaching for curl commands"
          
  - id: custom-validation-script
    name: "Run custom validation for deployments"
    matcher:
      pattern: "^(kubectl apply|terraform apply)"
    actions:
      - script:
          path: "~/.substrate/hooks/validate-deployment.sh"
          mode: before
          context: environment
          timeout: 10s
          on_failure: block
          
  - id: ai-safety-check
    name: "AI command safety validation"
    conditions:
      - environment_var: "CLAUDE_CODE_SESSION"
    matcher:
      pattern: ".*"  # All commands from AI
    actions:
      - script:
          path: "/usr/local/bin/ai-safety-check"
          mode: wrap
          context: combined
          on_failure: block
          # Script can analyze, log, modify, or block based on ML models
          
  - id: notify-on-production
    name: "Send notifications for production commands"
    matcher:
      pattern: "production|prod"
    actions:
      - script:
          path: "~/.substrate/hooks/notify.py"
          mode: async  # Don't block command execution
          context: stdin
          # Script handles Slack, email, PagerDuty notifications
```

### Phase 2: Hook Evaluation Engine (Week 2-3)

#### 2.1 Hook Evaluator Module

```rust
// crates/hooks/src/evaluator.rs
pub struct HookEvaluator {
    hooks: Vec<Hook>,
    cache: LruCache<String, EvalResult>,
    metrics: Metrics,
}

impl HookEvaluator {
    pub fn evaluate(&mut self, context: &CommandContext) -> EvalResult {
        // Check cache first
        if let Some(cached) = self.cache.get(&context.cache_key()) {
            return cached.clone();
        }
        
        // Evaluate hooks in priority order
        let mut applicable_hooks = self.hooks
            .iter()
            .filter(|h| h.enabled && h.matches(context))
            .collect::<Vec<_>>();
            
        applicable_hooks.sort_by_key(|h| -h.priority);
        
        for hook in applicable_hooks {
            if let Some(result) = self.evaluate_hook(hook, context) {
                self.cache.put(context.cache_key(), result.clone());
                return result;
            }
        }
        
        EvalResult::Allow
    }
}
```

#### 2.2 Integration with Shim

```rust
// Modify crates/shim/src/exec.rs
pub fn run_shim() -> Result<i32> {
    // ... existing early checks ...
    
    let ctx = ShimContext::from_current_exe()?;
    
    // NEW: Hook evaluation point
    let hook_result = if let Some(evaluator) = load_hook_evaluator()? {
        let cmd_context = CommandContext::from_shim(&ctx)?;
        evaluator.evaluate(&cmd_context)
    } else {
        EvalResult::Allow
    };
    
    match hook_result {
        EvalResult::Block(config) => {
            log_blocked_command(&ctx, &config)?;
            eprintln!("{}", config.message);
            return Ok(config.exit_code);
        }
        EvalResult::Modify(config) => {
            // Modify command/arguments
            ctx.apply_modifications(config)?;
        }
        EvalResult::Prompt(config) => {
            if !prompt_user(&config)? {
                return Ok(1);
            }
        }
        _ => {} // Continue with other actions
    }
    
    // ... continue with existing execution ...
}
```

### Phase 3: Advanced Features (Week 3-4)

#### 3.1 Context-Aware Hooks

```rust
pub struct CommandContext {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: PathBuf,
    pub session_id: String,
    pub parent_cmd_id: Option<String>,
    pub depth: u32,
    pub caller_chain: Vec<String>,
    pub user: String,
    pub groups: Vec<String>,
    pub timestamp: SystemTime,
    pub tty_info: Option<TtyInfo>,
    pub binary_hash: Option<String>,
}
```

#### 3.2 Hook Actions Implementation

**Block Action**
```rust
impl BlockAction {
    fn execute(&self, context: &CommandContext) -> Result<EvalResult> {
        // Log the block event
        log_event(LogLevel::Security, "command_blocked", json!({
            "command": context.command,
            "reason": self.reason,
            "policy": self.policy_id,
        }))?;
        
        // Send telemetry if configured
        if let Some(telemetry) = &self.telemetry {
            telemetry.send_blocked_command(context)?;
        }
        
        Ok(EvalResult::Block(self.config.clone()))
    }
}
```

**Modify Action**
```rust
impl ModifyAction {
    fn execute(&self, context: &mut CommandContext) -> Result<()> {
        match &self.modification {
            Modification::ReplaceCommand(new_cmd) => {
                context.command = new_cmd.clone();
            }
            Modification::AddArgument(arg) => {
                context.args.push(arg.clone());
            }
            Modification::RemoveArgument(pattern) => {
                context.args.retain(|a| !pattern.is_match(a));
            }
            Modification::InjectEnvVar(key, value) => {
                context.env.insert(key.clone(), value.clone());
            }
            Modification::WrapCommand(wrapper) => {
                // Transform: cmd args -> wrapper cmd args
                let original = context.command.clone();
                context.command = wrapper.clone();
                context.args.insert(0, original);
            }
        }
        Ok(())
    }
}
```

**Sandbox Action**
```rust
impl SandboxAction {
    fn execute(&self, context: &CommandContext) -> Result<()> {
        // Set up restricted environment
        context.env.insert("SUBSTRATE_SANDBOX".to_string(), "1".to_string());
        
        // Restrict PATH to safe directories
        if self.restrict_path {
            let safe_path = "/usr/local/bin:/usr/bin:/bin";
            context.env.insert("PATH".to_string(), safe_path.to_string());
        }
        
        // Set resource limits
        if let Some(limits) = &self.resource_limits {
            context.env.insert("SUBSTRATE_RLIMIT_CPU".to_string(), 
                             limits.cpu_seconds.to_string());
            context.env.insert("SUBSTRATE_RLIMIT_MEM".to_string(), 
                             limits.memory_mb.to_string());
        }
        
        Ok(())
    }
}
```

**Script Action** *(Critical for extensibility - similar to Claude Code hooks)*
```rust
pub struct ScriptConfig {
    pub script_path: PathBuf,           // Path to script/executable
    pub pass_context: ContextPassMode,  // How to pass context to script
    pub timeout: Option<Duration>,      // Execution timeout
    pub run_mode: ScriptRunMode,        // When to run the script
    pub failure_action: FailureAction,  // What to do if script fails
}

pub enum ContextPassMode {
    Environment,     // Pass as env vars (HOOK_COMMAND, HOOK_ARGS, etc.)
    Stdin,          // Pass as JSON on stdin
    Arguments,      // Pass as command-line arguments
    Combined,       // All of the above
}

pub enum ScriptRunMode {
    Before,         // Run before the command (can block/modify)
    After,          // Run after the command completes
    Replace,        // Run instead of the command
    Wrap,           // Wrap the command execution
    Async,          // Run asynchronously (fire-and-forget)
}

pub enum FailureAction {
    Block,          // Block the original command
    Warn,           // Warn but continue
    Ignore,         // Silently continue
    Retry(u32),     // Retry the script N times
}

impl ScriptAction {
    fn execute(&self, context: &mut CommandContext) -> Result<ScriptResult> {
        // Prepare script environment
        let mut script_env = self.prepare_environment(context)?;
        
        // Build script command
        let mut cmd = Command::new(&self.script_path);
        
        // Pass context based on mode
        match self.pass_context {
            ContextPassMode::Environment => {
                cmd.env("HOOK_COMMAND", &context.command);
                cmd.env("HOOK_ARGS", context.args.join(" "));
                cmd.env("HOOK_CWD", &context.cwd.display().to_string());
                cmd.env("HOOK_SESSION_ID", &context.session_id);
                cmd.env("HOOK_DEPTH", context.depth.to_string());
                cmd.env("HOOK_USER", &context.user);
                
                // Pass the original command that would be executed
                if let Some(binary_path) = &context.resolved_binary {
                    cmd.env("HOOK_BINARY_PATH", binary_path.display().to_string());
                }
            }
            ContextPassMode::Stdin => {
                let json = serde_json::to_string(&context)?;
                cmd.stdin(Stdio::piped());
                // Will write JSON after spawn
            }
            ContextPassMode::Arguments => {
                cmd.arg(&context.command);
                cmd.args(&context.args);
            }
            ContextPassMode::Combined => {
                // Do all three
                self.apply_all_modes(&mut cmd, context)?;
            }
        }
        
        // Handle different run modes
        match self.run_mode {
            ScriptRunMode::Before => {
                let output = self.run_with_timeout(cmd)?;
                self.process_script_output(output, context)?
            }
            ScriptRunMode::Replace => {
                // Script replaces the original command entirely
                let output = self.run_with_timeout(cmd)?;
                if output.status.success() {
                    // Prevent original command from running
                    return Ok(ScriptResult::ReplaceCommand);
                }
            }
            ScriptRunMode::Wrap => {
                // Script wraps the command - it's responsible for executing it
                cmd.env("HOOK_WRAPPED_COMMAND", format!("{} {}", 
                    context.command, context.args.join(" ")));
                let output = self.run_with_timeout(cmd)?;
                return Ok(ScriptResult::Wrapped(output.status));
            }
            ScriptRunMode::Async => {
                // Fire and forget - don't wait for completion
                cmd.spawn()?;
                return Ok(ScriptResult::AsyncStarted);
            }
            ScriptRunMode::After => {
                // Will be executed after the main command
                return Ok(ScriptResult::DeferredScript(cmd));
            }
        }
        
        Ok(ScriptResult::Continue)
    }
    
    fn process_script_output(&self, output: Output, context: &mut CommandContext) -> Result<()> {
        // Scripts can output JSON to modify the command
        if output.stdout.starts_with(b"{") {
            if let Ok(modifications) = serde_json::from_slice::<CommandModifications>(&output.stdout) {
                if let Some(new_cmd) = modifications.command {
                    context.command = new_cmd;
                }
                if let Some(new_args) = modifications.args {
                    context.args = new_args;
                }
                if let Some(env_vars) = modifications.env {
                    context.env.extend(env_vars);
                }
                if let Some(block) = modifications.block {
                    if block {
                        return Err(anyhow!("Script requested blocking: {}", 
                            modifications.message.unwrap_or_default()));
                    }
                }
            }
        }
        Ok(())
    }
}
```

### Phase 4: Hook Management CLI (Week 4)

#### 4.1 CLI Commands

```bash
# List all hooks
substrate hooks list [--enabled-only] [--format=table|json|yaml]

# Add a new hook
substrate hooks add --name "Block sudo" --pattern "^sudo" --action block

# Enable/disable hooks
substrate hooks enable <hook-id>
substrate hooks disable <hook-id>

# Test hook matching
substrate hooks test "git push --force" [--context context.json]

# Import/export hooks
substrate hooks import hooks.yaml
substrate hooks export --output=hooks-backup.yaml

# Hook statistics
substrate hooks stats [--period=1h|24h|7d]
```

#### 4.2 Hook Management API

```rust
// crates/hooks/src/management.rs
pub struct HookManager {
    config_path: PathBuf,
    hooks: Vec<Hook>,
}

impl HookManager {
    pub fn add_hook(&mut self, hook: Hook) -> Result<()> {
        self.validate_hook(&hook)?;
        self.check_conflicts(&hook)?;
        self.hooks.push(hook);
        self.persist()?;
        Ok(())
    }
    
    pub fn validate_hook(&self, hook: &Hook) -> Result<()> {
        // Validate regex patterns
        if let Matcher::Pattern(pattern) = &hook.matcher {
            regex::Regex::new(pattern)?;
        }
        
        // Validate priority range
        if hook.priority < -1000 || hook.priority > 1000 {
            return Err(anyhow!("Priority must be between -1000 and 1000"));
        }
        
        // Validate actions don't conflict
        self.validate_actions(&hook.actions)?;
        
        Ok(())
    }
}
```

## 🔒 Security Considerations

### 1. Hook Bypass Prevention

```rust
// Prevent hooks from being bypassed via PATH manipulation
fn validate_no_path_injection(context: &CommandContext) -> Result<()> {
    // Check for suspicious PATH entries
    if let Some(path) = context.env.get("PATH") {
        for entry in path.split(':') {
            if entry == "." || entry.starts_with("./") || entry.starts_with("../") {
                return Err(anyhow!("Suspicious PATH entry detected"));
            }
        }
    }
    Ok(())
}
```

### 2. Hook Configuration Security

- Hook configuration files must have restrictive permissions (0600)
- Hooks can only be modified by the owner user
- Configuration changes trigger audit log entries
- Cryptographic signing of hook configurations (optional)

### 3. Rate Limiting & DoS Prevention

```rust
pub struct RateLimiter {
    buckets: HashMap<String, TokenBucket>,
}

impl RateLimiter {
    pub fn check_rate_limit(&mut self, key: &str, limit: &RateLimit) -> bool {
        let bucket = self.buckets.entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(limit));
        bucket.try_consume(1)
    }
}
```

### 4. Audit Trail

```rust
pub struct HookAuditLog {
    pub timestamp: SystemTime,
    pub hook_id: String,
    pub action: String,
    pub command: String,
    pub result: EvalResult,
    pub session_id: String,
    pub user: String,
}
```

## 📜 Script Hook Examples

### Example: Python Safety Validator
```python
#!/usr/bin/env python3
# ~/.substrate/hooks/validate-command.py

import json
import sys
import os
import re

def main():
    # Get context from environment or stdin
    command = os.environ.get('HOOK_COMMAND', '')
    args = os.environ.get('HOOK_ARGS', '').split()
    session_id = os.environ.get('HOOK_SESSION_ID', '')
    user = os.environ.get('HOOK_USER', '')
    
    # Custom validation logic
    dangerous_patterns = [
        r'rm.*-rf.*/',           # Recursive root deletion
        r':(){ :|:& };:',       # Fork bomb
        r'dd.*if=/dev/zero',    # Disk wipe
        r'chmod.*777.*/',       # Overly permissive permissions
    ]
    
    full_command = f"{command} {' '.join(args)}"
    
    for pattern in dangerous_patterns:
        if re.search(pattern, full_command):
            # Output JSON to block command
            result = {
                "block": True,
                "message": f"Dangerous command pattern detected: {pattern}",
                "log_level": "security",
                "notify": ["security-team@company.com"]
            }
            print(json.dumps(result))
            sys.exit(0)
    
    # Check if AI is trying to modify system files
    if 'CLAUDE_CODE' in os.environ:
        if any(path.startswith('/etc') or path.startswith('/usr') for path in args):
            result = {
                "block": True,
                "message": "AI agents cannot modify system files",
                "suggest": "Consider using a local development environment"
            }
            print(json.dumps(result))
            sys.exit(0)
    
    # Allow with modifications
    if command == 'curl' and '-k' in args:
        result = {
            "args": [arg for arg in args if arg != '-k'],  # Remove insecure flag
            "message": "Removed insecure -k flag from curl",
            "log_level": "warning"
        }
        print(json.dumps(result))
    
    sys.exit(0)  # Success - allow command
```

### Example: Bash Wrapper Script
```bash
#!/bin/bash
# ~/.substrate/hooks/wrap-git-commit.sh

# This script wraps git commit commands to ensure quality

# Get the wrapped command from environment
WRAPPED_CMD="$HOOK_WRAPPED_COMMAND"

# Run pre-commit checks
echo "Running pre-commit checks..."
if ! pre-commit run --all-files; then
    echo "Pre-commit checks failed. Fix issues and try again."
    exit 1
fi

# Check for secret scanning
if ! git secrets --scan; then
    echo "Detected secrets in commit. Aborting."
    exit 1
fi

# Execute the original command
echo "Checks passed. Proceeding with commit..."
eval "$WRAPPED_CMD"
EXIT_CODE=$?

# Post-commit actions
if [ $EXIT_CODE -eq 0 ]; then
    # Log successful commit
    echo "{\"event\": \"commit_success\", \"user\": \"$HOOK_USER\"}" >> ~/.substrate/commit.log
    
    # Trigger CI if on main branch
    if [ "$(git branch --show-current)" = "main" ]; then
        curl -X POST https://ci.company.com/trigger
    fi
fi

exit $EXIT_CODE
```

### Example: Node.js Async Notifier
```javascript
#!/usr/bin/env node
// ~/.substrate/hooks/notify-slack.js

const https = require('https');

// Get context from environment
const command = process.env.HOOK_COMMAND;
const args = process.env.HOOK_ARGS;
const user = process.env.HOOK_USER;
const sessionId = process.env.HOOK_SESSION_ID;

// Slack webhook URL
const SLACK_WEBHOOK = 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL';

// Build message
const message = {
    text: `Command executed by ${user}`,
    blocks: [
        {
            type: 'section',
            text: {
                type: 'mrkdwn',
                text: `*Command:* \`${command} ${args}\`\n*User:* ${user}\n*Session:* ${sessionId}`
            }
        }
    ]
};

// Send to Slack (async - don't wait)
const req = https.request(SLACK_WEBHOOK, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' }
});

req.write(JSON.stringify(message));
req.end();

// Exit immediately (async mode)
process.exit(0);
```

## 🚀 Potential Features & Use Cases

### 1. AI Agent Safety Controls

```yaml
# Prevent AI agents from modifying critical files
- id: protect-system-files
  matcher:
    pattern: "^(rm|mv|chmod|chown).*/(etc|usr|bin|sbin)"
  conditions:
    - environment_var: "CLAUDE_CODE_SESSION"
  actions:
    - block:
        message: "AI agents cannot modify system files"
```

### 2. Compliance & Auditing

```yaml
# Log all database access for compliance
- id: audit-database-access
  matcher:
    pattern: "^(psql|mysql|mongo|redis-cli)"
  actions:
    - log:
        level: "audit"
        include_args: true
        include_env: ["USER", "SSH_CLIENT"]
    - notify:
        syslog: true
        facility: "auth"
```

### 3. Development Workflow Enhancement

```yaml
# Auto-formatting before git commits
- id: format-before-commit
  matcher:
    pattern: "^git commit"
  actions:
    - modify:
        prepend_command: "cargo fmt && cargo clippy --fix &&"
```

### 4. Security Scanning Integration

```yaml
# Scan packages before installation
- id: scan-npm-packages
  matcher:
    exact: "npm install"
  actions:
    - modify:
        wrap_command: "substrate-security-scan npm"
    - prompt:
        condition: "high_risk_packages_detected"
        message: "High risk packages detected. Continue?"
```

### 5. Learning Mode

```yaml
# Learn normal behavior patterns
- id: learning-mode
  enabled: true
  matcher:
    pattern: ".*"
  actions:
    - log:
        mode: "learning"
        capture_patterns: true
        build_baseline: true
    - analyze:
        detect_anomalies: true
        alert_on_deviation: true
```

### 6. Multi-Stage Approval Workflows

```yaml
# Require approval for production deployments
- id: production-deployment
  matcher:
    pattern: "^(kubectl|terraform|ansible).*production"
  actions:
    - prompt:
        message: "Production deployment detected"
        require_2fa: true
    - notify:
        slack_channel: "#deployments"
        include_approval_link: true
    - wait_for_approval:
        timeout: 5m
        approvers: ["@devops-team"]
```

## 📊 Performance Considerations

### 1. Hook Evaluation Caching

- LRU cache for hook evaluation results
- Cache key based on command + args + relevant context
- TTL-based expiration for time-sensitive conditions
- Approximately 2-3ms overhead for cached evaluations

### 2. Async Action Processing

```rust
// Non-blocking notifications and logging
pub async fn process_async_actions(actions: Vec<AsyncAction>) {
    let handles: Vec<_> = actions.into_iter()
        .map(|action| tokio::spawn(async move {
            action.execute().await
        }))
        .collect();
    
    // Don't wait for async actions to complete
    for handle in handles {
        tokio::spawn(async move {
            let _ = handle.await;
        });
    }
}
```

### 3. Hook Compilation & Optimization

```rust
// Pre-compile regex patterns at load time
pub struct CompiledHook {
    pub hook: Hook,
    pub compiled_matcher: CompiledMatcher,
    pub compiled_conditions: Vec<CompiledCondition>,
}
```

## 🔄 Migration Strategy

### Phase 1: Opt-in Beta
- Feature flag: `SUBSTRATE_HOOKS_ENABLED=1`
- Default hooks ship disabled
- Extensive logging in "dry-run" mode

### Phase 2: Progressive Rollout
- Enable for specific commands first
- Gradual expansion based on telemetry
- A/B testing for performance impact

### Phase 3: General Availability
- Hooks enabled by default (with bypass option)
- Migration tools for existing policies
- Comprehensive documentation and examples

## 📈 Success Metrics

1. **Performance Impact**: < 5ms p99 latency addition
2. **Adoption Rate**: 50% of users configure custom hooks within 3 months
3. **Security Events**: 90% reduction in accidental dangerous commands
4. **AI Safety**: 100% of high-risk AI commands intercepted
5. **User Satisfaction**: > 4.5/5 rating for hook system

## 🏁 Conclusion

The Substrate Command Hook System transforms command interception from passive observation to active mediation, providing unprecedented control over command execution. This enables:

- **Enhanced Security**: Block dangerous commands before execution
- **AI Safety**: Control and audit AI agent behavior
- **Compliance**: Enforce organizational policies automatically
- **Developer Experience**: Streamline workflows with intelligent automation
- **Observability**: Rich context for debugging and analysis

The modular architecture ensures the system can evolve with user needs while maintaining performance and security. The hook system positions Substrate as not just a command tracer, but a comprehensive command execution platform for the age of AI-assisted development.