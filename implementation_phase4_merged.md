# Phase 4: Security, Agent API, and Graph Intelligence (Merged Plan)

## Overview

Phase 4 adds security enforcement, AI agent integration, and graph-based intelligence to substrate WITHOUT rewriting existing functionality. Everything is additive and can be enabled via environment variables. **Now with cross-platform support via world backend abstraction.**

## World Backend Abstraction (Cross-Platform Foundation)

```rust
// crates/world-api/src/lib.rs
use anyhow::Result;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldSpec {
    pub reuse_session: bool,
    pub isolate_network: bool,
    pub limits: ResourceLimits,
    pub enable_preload: bool,
    pub allowed_domains: Vec<String>,   // egress allowlist (to be resolved → IP set)
    pub project_dir: PathBuf,           // host project dir
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu: Option<f32>,   // 2.0 = 2 CPUs
    pub mem: Option<String> // "2Gi"
}

pub struct WorldHandle {
    pub id: String,
}

pub struct ExecRequest {
    pub cmd: String,
    pub cwd: PathBuf,
    pub env: std::collections::HashMap<String, String>,
    pub pty: bool,
}

pub struct ExecResult {
    pub exit: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub scopes_used: Vec<String>,
}

pub trait WorldBackend: Send + Sync {
    fn ensure_session(&self, spec: &WorldSpec) -> Result<WorldHandle>;
    fn exec(&self, world: &WorldHandle, req: ExecRequest) -> Result<ExecResult>;
    fn fs_diff(&self, world: &WorldHandle, span_id: &str) -> Result<FsDiff>;
    fn apply_policy(&self, world: &WorldHandle, spec: &WorldSpec) -> Result<()>;
}

pub enum Backend {
    LinuxLocal,       // namespaces/cgroups/nftables (native Linux)
    MacLima,          // macOS Phase 4 backend (Linux VM + agent)
    LinuxDocker,      // optional fallback
    LinuxFirecracker, // optional: high-isolation ephemeral (Linux-only)
    WindowsWSL2,      // deferred
}
```

> **Why:** broker/trace/API become backend-agnostic. Linux & mac share the same enforcement logic.

## Top 5 Priorities (Ship These First)

### 1. Session World (Reusable by Default)

Run the user's real shell *inside* a reusable Linux world with proper isolation.

**Implementation:**
```rust
// crates/world/src/lib.rs
use anyhow::{Result, Context};
use nix::mount::{mount, MsFlags};
use nix::sched::{unshare, CloneFlags};
use std::path::PathBuf;

pub struct SessionWorld {
    pub id: String,
    pub root_dir: PathBuf,
    pub project_dir: PathBuf,
    pub cgroup_path: PathBuf,
    pub net_namespace: Option<String>,
}

impl SessionWorld {
    pub fn ensure_started(spec: WorldSpec) -> Result<Self> {
        // Check if session world already exists
        if let Some(existing) = Self::find_existing()? {
            return Ok(existing);
        }
        
        // Create new session world
        let world = Self {
            id: format!("wld_{}", uuid::Uuid::now_v7()),
            root_dir: PathBuf::from("/tmp/substrate-worlds"),
            project_dir: std::env::current_dir()?,
            cgroup_path: PathBuf::from("/sys/fs/cgroup/substrate"),
            net_namespace: None,
        };
        
        world.setup(spec)?;
        Ok(world)
    }
    
    fn setup(&self, spec: WorldSpec) -> Result<()> {
        // CRITICAL: Prevent mount propagation leaks from host
        mount(
            None::<&str>,
            "/",
            None::<&str>,
            MsFlags::MS_REC | MsFlags::MS_PRIVATE,
            None::<&str>,
        ).context("Failed to make mounts private")?;
        
        // Set up bind mounts
        self.setup_mounts()?;
        
        // Set up cgroups v2 limits (gracefully degrade if not available)
        if Path::new("/sys/fs/cgroup/cgroup.controllers").exists() {
            self.setup_cgroups(&spec.limits)?;
        } else {
            eprintln!("⚠️  cgroups v2 not available, skipping resource limits");
        }
        
        // Set up network namespace with nftables
        if spec.isolate_network {
            self.setup_network(&spec.allowed_hosts)?;
        }
        
        // User namespaces if available (fallback to Incus/Docker if not)
        if self.setup_user_namespace().is_err() {
            eprintln!("⚠️  User namespaces disabled, consider Incus/Docker fallback");
        }
        
        // Drop capabilities
        self.drop_capabilities()?;
        
        // Set no_new_privs
        prctl::set_no_new_privs()?;
        
        // Apply baseline seccomp filter (permissive + logging)
        self.apply_seccomp_baseline()?;
        
        Ok(())
    }
    
    fn setup_mounts(&self) -> Result<()> {
        // Project directory: read-write
        mount(
            Some(&self.project_dir),
            &self.root_dir.join("project"),
            None::<&str>,
            MsFlags::MS_BIND | MsFlags::MS_REC,
            None::<&str>,
        )?;
        
        // System directories: read-only (two-step for proper RO)
        for dir in &["/usr", "/bin", "/lib", "/lib64", "/etc"] {
            let target = self.root_dir.join(dir.trim_start_matches('/'));
            
            // Step 1: Bind mount
            mount(
                Some(dir),
                &target,
                None::<&str>,
                MsFlags::MS_BIND | MsFlags::MS_REC,
                None::<&str>,
            )?;
            
            // Step 2: Remount as read-only (required for actual RO)
            mount(
                None::<&str>,
                &target,
                None::<&str>,
                MsFlags::MS_REMOUNT | MsFlags::MS_BIND | MsFlags::MS_RDONLY,
                None::<&str>,
            )?;
        }
        
        // Pivot into the new root
        std::fs::create_dir_all(self.root_dir.join("old_root"))?;
        pivot_root(&self.root_dir, &self.root_dir.join("old_root"))?;
        
        // Mount clean /proc and minimal /dev
        mount("proc", "/proc", "proc", MsFlags::empty(), None::<&str>)?;
        self.setup_minimal_dev()?;
        
        // Unmount old root
        umount2("/old_root", MntFlags::MNT_DETACH)?;
        
        Ok(())
    }
    
    fn setup_minimal_dev(&self) -> Result<()> {
        // Create minimal /dev with only essential devices
        let devices = [("null", 1, 3), ("zero", 1, 5), ("urandom", 1, 9), ("tty", 5, 0)];
        
        for (name, major, minor) in devices {
            let path = Path::new("/dev").join(name);
            mknod(&path, SFlag::S_IFCHR, Mode::from_bits_truncate(0o666), makedev(major, minor))?;
        }
        
        Ok(())
    }
    
    fn setup_cgroups(&self, limits: &ResourceLimits) -> Result<()> {
        std::fs::create_dir_all(&self.cgroup_path)?;
        
        // Memory limit
        if let Some(mem) = &limits.memory {
            std::fs::write(
                self.cgroup_path.join("memory.max"),
                mem,
            )?;
        }
        
        // CPU limit (proper calculation: quota period format)
        if let Some(cpu) = &limits.cpu {
            let cpus = cpu.parse::<f64>().context("Invalid CPU limit")?;
            let quota = (cpus * 100000.0).round() as u64;
            std::fs::write(
                self.cgroup_path.join("cpu.max"),
                format!("{} 100000", quota),
            )?;
        }
        
        Ok(())
    }
    
    fn setup_user_namespace(&self) -> Result<()> {
        // Set up user namespace mapping
        unshare(CloneFlags::CLONE_NEWUSER)?;
        
        // Must write uid/gid maps before other namespace ops
        let uid = getuid();
        let gid = getgid();
        
        std::fs::write("/proc/self/setgroups", "deny")?;
        std::fs::write("/proc/self/uid_map", format!("0 {} 1", uid))?;
        std::fs::write("/proc/self/gid_map", format!("0 {} 1", gid))?;
        
        Ok(())
    }
    
    fn apply_seccomp_baseline(&self) -> Result<()> {
        // Baseline seccomp: deny dangerous syscalls, log violations
        use syscallz::{Context, Syscall, Action};
        
        let mut ctx = Context::init_with_action(Action::Allow)?;
        
        // Deny dangerous syscalls (log for now)
        let dangerous = [
            Syscall::mount,
            Syscall::umount2,
            Syscall::pivot_root,
            Syscall::keyctl,
            Syscall::perf_event_open,
            Syscall::bpf,
        ];
        
        for syscall in dangerous {
            ctx.set_action_for_syscall(Action::Log, syscall)?;
        }
        
        ctx.load()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct WorldSpec {
    pub reuse_session: bool,
    pub isolate_network: bool,
    pub allowed_hosts: Vec<String>,
    pub limits: ResourceLimits,
}

#[derive(Debug)]
pub struct ResourceLimits {
    pub cpu: Option<String>,     // "2" = 2 CPUs
    pub memory: Option<String>,  // "2Gi"
}
```

**Activation:**
```bash
export SUBSTRATE_WORLD=enabled
substrate  # Your shell now runs in a session world
```

**Accept:** 
- `pip install` can't write outside project
- Network to non-allowlisted hosts is blocked
- `ps aux` inside world cannot see host processes (PID namespace)
- Graceful degradation when cgroups v2 unavailable

**Network DNS Resolution:**
```rust
// crates/world/src/dns_resolver.rs
use std::net::IpAddr;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct DnsResolver {
    allowed_domains: Vec<String>,
    resolved_ips: RwLock<HashMap<String, CachedResolution>>,
}

struct CachedResolution {
    ips: Vec<IpAddr>,
    expires_at: Instant,
}

impl DnsResolver {
    pub fn spawn_resolver(allowed_domains: Vec<String>) -> Result<()> {
        let resolver = Arc::new(Self {
            allowed_domains,
            resolved_ips: RwLock::new(HashMap::new()),
        });
        
        // Spawn background task to refresh IPs
        std::thread::spawn(move || {
            loop {
                resolver.refresh_all().ok();
                std::thread::sleep(Duration::from_secs(60)); // Refresh every minute
            }
        });
        
        Ok(())
    }
    
    fn refresh_all(&self) -> Result<()> {
        let mut cache = self.resolved_ips.write()?;
        
        for domain in &self.allowed_domains {
            let ips = dns_lookup::lookup_host(domain)?;
            cache.insert(domain.clone(), CachedResolution {
                ips,
                expires_at: Instant::now() + Duration::from_secs(300), // 5 min TTL
            });
        }
        
        // Update nftables set atomically
        self.update_nftables_set(&cache)?;
        Ok(())
    }
    
    fn update_nftables_set(&self, cache: &HashMap<String, CachedResolution>) -> Result<()> {
        let mut all_ips = Vec::new();
        for resolution in cache.values() {
            all_ips.extend(&resolution.ips);
        }
        
        // Atomic set update
        let cmds = format!(
            "nft add set inet substrate allowed_ips {{ type ipv4_addr; flags interval; }}\n\
             nft flush set inet substrate allowed_ips\n\
             nft add element inet substrate allowed_ips {{ {} }}",
            all_ips.iter().map(|ip| ip.to_string()).collect::<Vec<_>>().join(", ")
        );
        
        Command::new("sh").arg("-c").arg(&cmds).output()?;
        Ok(())
    }
}

// Pin DNS inside world to controlled resolver
fn setup_dns_stub(&self) -> Result<()> {
    // Create resolv.conf pointing to our stub
    std::fs::write(
        self.root_dir.join("etc/resolv.conf"),
        "nameserver 127.0.0.53\noptions edns0 trust-ad\n"
    )?;
    
    // Start stub resolver on 127.0.0.53
    self.start_stub_resolver()?;
    Ok(())
}
```

**DNS & Egress Clarification:**
- Maintain a background **resolver task** that converts allowed domains → **IP set** in nftables
- Pin world DNS to `127.0.0.53` inside the world/VM to prevent bypass
- Update set atomically (flush then add) to avoid racey half-states
- TTL-aware rotation of IPs with configurable refresh interval

### 1b. macOS Support via Lima (Phase 4 Platform Parity)

**Goal:** Identical policy, enforcement, and API semantics on macOS without rewriting isolation.

#### Host-side Runner (MacLima Backend)

```rust
// crates/world-mac-lima/src/lib.rs
use crate::world_api::{WorldBackend, WorldHandle, WorldSpec, ExecRequest, ExecResult};

pub struct MacLimaBackend {
    vm_name: String,
    agent_socket: PathBuf,
}

impl MacLimaBackend {
    pub fn new() -> Result<Self> {
        Ok(Self {
            vm_name: "substrate".into(),
            agent_socket: home_dir().join(".substrate/sock/lima-agent.sock"),
        })
    }
    
    fn ensure_vm_running(&self) -> Result<()> {
        // Check if VM exists and is running
        let status = Command::new("limactl")
            .args(&["list", &self.vm_name, "--json"])
            .output()?;
        
        if !status.status.success() || !status.stdout.contains("Running") {
            // Start VM (idempotent)
            Command::new("limactl")
                .args(&["start", &self.vm_name, "--tty=false"])
                .status()?;
            
            // Wait for agent to be ready
            self.wait_for_agent()?;
        }
        
        Ok(())
    }
    
    fn setup_socket_forwarding(&self) -> Result<()> {
        // Option 1: VSock (preferred)
        // Option 2: SSH forwarding fallback
        Command::new("ssh")
            .args(&[
                "-N", "-L",
                &format!("{}:/run/substrate.sock", self.agent_socket.display()),
                &format!("lima:{}", self.vm_name),
            ])
            .spawn()?;
        
        Ok(())
    }
}

impl WorldBackend for MacLimaBackend {
    fn ensure_session(&self, spec: &WorldSpec) -> Result<WorldHandle> {
        self.ensure_vm_running()?;
        self.setup_socket_forwarding()?;
        
        // Forward to agent inside VM
        let client = UnixClient::connect(&self.agent_socket)?;
        let response = client.call("ensure_session", spec)?;
        
        Ok(WorldHandle { id: response.world_id })
    }
    
    fn exec(&self, world: &WorldHandle, req: ExecRequest) -> Result<ExecResult> {
        let client = UnixClient::connect(&self.agent_socket)?;
        client.call("exec", (world, req))
    }
}
```

#### Guest-side Agent (Runs Inside VM)

```rust
// crates/world-agent/src/main.rs
use substrate_world::{SessionWorld, WorldSpec};  // Reuse Linux code!

#[tokio::main]
async fn main() -> Result<()> {
    let socket_path = Path::new("/run/substrate.sock");
    let listener = UnixListener::bind(socket_path)?;
    
    // Start systemd unit
    systemd::daemon::notify(false, &[("READY", "1")])?;
    
    let app = Router::new()
        .route("/ensure_session", post(ensure_session))
        .route("/exec", post(execute_command))
        .route("/fs_diff", post(compute_diff))
        .route("/apply_policy", post(apply_policy));
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ensure_session(Json(spec): Json<WorldSpec>) -> Result<Json<WorldHandle>> {
    // THIS IS THE KEY: Reuse all Linux isolation code!
    let world = SessionWorld::ensure_started(spec)?;
    Ok(Json(WorldHandle { id: world.id }))
}

async fn execute_command(Json(req): Json<(WorldHandle, ExecRequest)>) -> Result<Json<ExecResult>> {
    let world = SessionWorld::from_handle(&req.0)?;
    let result = world.execute(&req.1.cmd, &req.1.cwd, req.1.env)?;
    
    Ok(Json(ExecResult {
        exit: result.exit_code,
        stdout: result.stdout,
        stderr: result.stderr,
        scopes_used: result.scopes_used,
    }))
}
```

#### Lima Configuration

```yaml
# ~/.lima/substrate.yaml
images:
  - location: "https://cloud-images.ubuntu.com/noble/current/noble-server-cloudimg-arm64.img"
    arch: "aarch64"
  - location: "https://cloud-images.ubuntu.com/noble/current/noble-server-cloudimg-amd64.img"
    arch: "x86_64"
    
mounts:
  - location: "~"
    writable: false  # Home directory read-only
  - location: "$PROJECT"
    writable: true   # Project directory read-write (virtiofs)
    
containerd: false

ssh:
  loadDotSSHPubKeys: false
  forwardAgent: false
  
provision:
  - mode: system
    script: |
      #!/bin/sh
      set -e
      apt-get update -y
      apt-get install -y nftables ca-certificates curl jq git build-essential
      
      # Enable user namespaces
      sysctl -w kernel.unprivileged_userns_clone=1 || true
      echo "kernel.unprivileged_userns_clone=1" >> /etc/sysctl.d/99-substrate.conf
      
      # Enable nftables
      systemctl enable nftables || true
      
      # Create substrate directories
      mkdir -p /var/lib/substrate/overlay
      mkdir -p /run/substrate
      
      # world-agent will be installed by host on first run

network:
  vnl: true  # Virtual network

# Optimization for performance
cpus: 2
memory: "4GiB"
disk: "20GiB"
```

#### Systemd Unit for Agent

```ini
# /etc/systemd/system/substrate-agent.service
[Unit]
Description=Substrate World Agent
After=network.target

[Service]
Type=notify
ExecStart=/usr/local/bin/substrate-world-agent
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

# Security
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/lib/substrate /run/substrate

[Install]
WantedBy=multi-user.target
```

#### Host ↔ VM Transport

```rust
// Transport options in order of preference
enum Transport {
    VSock,      // Fastest, if available
    UnixSocket, // SSH forwarded
    TCP,        // Last resort
}

impl Transport {
    fn auto_select() -> Result<Self> {
        // Try VSock first (macOS 13+)
        if std::path::Path::new("/dev/vsock").exists() {
            return Ok(Self::VSock);
        }
        
        // Fall back to SSH forwarding
        Ok(Self::UnixSocket)
    }
}
```

**Egress Budget:**
```rust
pub struct EgressBudget {
    bytes_per_sec: Option<u64>,
    total_bytes: Option<u64>,
    used_bytes: AtomicU64,
}

impl Broker {
    pub fn check_egress(&self, bytes: u64) -> Result<Decision> {
        if let Some(budget) = &self.egress_budget {
            let used = budget.used_bytes.fetch_add(bytes, Ordering::SeqCst);
            
            if let Some(total) = budget.total_bytes {
                if used + bytes > total {
                    trace::rate_limit(bytes, used, total)?;
                    return Ok(Decision::Deny("Egress budget exceeded".into()));
                }
            }
        }
        Ok(Decision::Allow)
    }
}

### 2. Broker in the Hot Path

Gate every execution with policy evaluation.

**Implementation:**
```rust
// crates/broker/src/lib.rs
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    
    // Filesystem
    pub fs_read: Vec<String>,   // Paths that can be read
    pub fs_write: Vec<String>,  // Paths that can be written
    
    // Network
    pub net_allowed: Vec<String>,  // Allowed hosts/domains
    
    // Commands
    pub cmd_allowed: Vec<String>,   // Allowed command patterns
    pub cmd_denied: Vec<String>,    // Denied command patterns
    pub cmd_isolated: Vec<String>,  // Commands to run in isolated world
    
    // Behavior
    pub require_approval: bool,
    pub allow_shell_operators: bool,
}

pub struct Broker {
    policy: Arc<Policy>,  // Atomic swaps for reload
    approvals: RwLock<ApprovalCache>,
    observe_only: AtomicBool,
}

#[derive(Debug)]
pub enum Decision {
    Allow,
    AllowWithRestrictions(Vec<Restriction>),
    Deny(String),
}

#[derive(Debug)]
pub struct Restriction {
    pub type_: RestrictionType,
    pub value: String,
}

#[derive(Debug)]
pub enum RestrictionType {
    IsolatedWorld,
    OverlayFS,
    NetworkFilter,
}

impl Broker {
    pub fn evaluate(cmd: &str, cwd: &str, world_id: &str) -> Result<Decision> {
        let broker = GLOBAL_BROKER.read()?;
        let policy = broker.policy.read()?;
        
        // Check denied commands first
        for pattern in &policy.cmd_denied {
            if matches_pattern(cmd, pattern) {
                trace::violation(cmd, "Command explicitly denied")?;
                return Ok(Decision::Deny("Command explicitly denied".into()));
            }
        }
        
        // Check if allowed
        let mut allowed = false;
        for pattern in &policy.cmd_allowed {
            if matches_pattern(cmd, pattern) {
                allowed = true;
                break;
            }
        }
        
        if !allowed && !policy.cmd_allowed.is_empty() {
            trace::violation(cmd, "Command not in allowlist")?;
            return Ok(Decision::Deny("Command not explicitly allowed".into()));
        }
        
        // Check if needs isolation
        for pattern in &policy.cmd_isolated {
            if matches_pattern(cmd, pattern) {
                return Ok(Decision::AllowWithRestrictions(vec![
                    Restriction {
                        type_: RestrictionType::IsolatedWorld,
                        value: "ephemeral".into(),
                    }
                ]));
            }
        }
        
        // Check if approval required
        if policy.require_approval {
            let approval = broker.check_approval(cmd)?;
            match approval {
                ApprovalStatus::Approved => {},
                ApprovalStatus::Denied => {
                    return Ok(Decision::Deny("User denied approval".into()));
                },
                ApprovalStatus::Unknown => {
                    let approved = broker.request_approval(cmd)?;
                    if !approved {
                        return Ok(Decision::Deny("User denied approval".into()));
                    }
                }
            }
        }
        
        Ok(Decision::Allow)
    }
    
    pub fn quick_check(argv: &[String], cwd: &str) -> Result<Decision> {
        // Fast path for shims - just check deny list
        let cmd = argv.join(" ");
        let policy = GLOBAL_BROKER.read()?.policy.read()?;
        
        for pattern in &policy.cmd_denied {
            if matches_pattern(&cmd, pattern) {
                return Ok(Decision::Deny("Command denied by policy".into()));
            }
        }
        
        Ok(Decision::Allow)
    }
}

// Interactive approval with diff preview
pub fn request_approval(cmd: &str, context: &ApprovalContext) -> Result<bool> {
    use dialoguer::{theme::ColorfulTheme, Select};
    use colored::*;
    
    println!("\n{}", "Command Approval Request".yellow().bold());
    println!("Command: {}", cmd.white());
    
    // Show diff preview when available
    if let Some(preview) = &context.diff_preview {
        println!("Impact: {}", preview.dim());
    }
    
    let options = vec![
        "Allow once",
        "Allow for session",
        "Allow always",
        "Deny",
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&options)
        .default(0)
        .interact()?;
    
    let cache_key = ApprovalCacheKey {
        policy_id: context.policy_id.clone(),
        cwd_prefix: context.cwd.clone(),
        command_pattern: extract_pattern(cmd),
    };
    
    match selection {
        0 => Ok(true),  // Once
        1 => {
            APPROVAL_CACHE.add_session(cache_key);
            Ok(true)
        },
        2 => {
            APPROVAL_CACHE.add_always(cache_key);
            Ok(true)
        },
        3 => Ok(false),  // Deny
        _ => Ok(false),
    }
}

// Atomic policy reload
pub fn reload_policy(new_policy: Policy) -> Result<()> {
    let broker = GLOBAL_BROKER.write()?;
    broker.policy = Arc::new(new_policy);
    
    // Clear approval cache on profile change
    broker.approvals.write()?.clear();
    
    eprintln!("✓ Policy reloaded atomically");
    Ok(())
}

// Observe-only mode for gradual rollout
impl Broker {
    pub fn set_mode(&self, mode: PolicyMode) {
        match mode {
            PolicyMode::Observe => self.observe_only.store(true, Ordering::SeqCst),
            PolicyMode::Enforce => self.observe_only.store(false, Ordering::SeqCst),
        }
    }
    
    pub fn evaluate_with_mode(&self, cmd: &str) -> Result<Decision> {
        let decision = self.evaluate(cmd)?;
        
        if self.observe_only.load(Ordering::SeqCst) {
            if matches!(decision, Decision::Deny(_)) {
                trace::policy_violation(cmd, "would_deny", &decision)?;
                return Ok(Decision::Allow);  // Observe but don't block
            }
        }
        
        Ok(decision)
    }
}
```

**Integration in shell:**
```rust
// crates/shell/src/lib.rs
let decision = broker::evaluate(&cmd_line, &cwd, &WORLD_ID)?;
match decision {
    Decision::Allow => execute_command(cmd)?,
    Decision::AllowWithRestrictions(r) => execute_with_restrictions(cmd, r)?,
    Decision::Deny(msg) => {
        eprintln!("🔒 {}", msg);
        return Ok(126);
    }
}
```

**Integration in shim:**
```rust
// crates/shim/src/main.rs
if env::var("SHIM_BYPASS") != Ok("1") {
    if let Decision::Deny(msg) = broker::quick_check(&argv, &cwd)? {
        eprintln!("🔒 {}", msg);
        std::process::exit(126);
    }
}
```

**Accept:** `curl https://random.com` is denied with reason; `--require-approval` prompts user.

### 3. Unified Agent-Trace Spans

Extend JSONL with stable span schema for agent correlation.

**Schema:**
```json
{
  "ts": "2024-01-01T00:00:00Z",
  "event_type": "command_complete",
  "session_id": "ses_xxx",
  "span_id": "spn_xxx",
  "parent_span": "spn_yyy",
  "component": "shell|shim|broker|world",
  "world_id": "wld_xxx",
  "policy_id": "default",
  "agent_id": "human|claude|cursor|qwen",
  "cwd": "/projects/foo",
  "cmd": "npm install",
  "exit": 0,
  "scopes_used": ["fs.write:/projects/foo/node_modules", "net:registry.npmjs.org:443"],
  "fs_diff": {
    "writes": ["node_modules/..."],
    "mods": ["package-lock.json"],
    "deletes": []
  }
}
```

**Helpers:**
```rust
// crates/trace/src/lib.rs
pub fn new_span(parent: Option<&str>) -> String {
    format!("spn_{}", uuid::Uuid::now_v7())
}

pub fn finish_span(span_id: &str, exit_code: i32, scopes: Vec<String>, diff: FsDiff) -> Result<()> {
    // Capture replay context for determinism
    let replay_context = ReplayContext {
        path: env::var("PATH").ok(),
        env_hash: hash_env_vars()?,
        umask: get_umask()?,
        locale: env::var("LANG").ok(),
        cwd: env::current_dir()?.to_string_lossy().to_string(),
        policy_id: CURRENT_POLICY_ID.read()?.clone(),
        policy_commit: get_policy_git_hash()?,
        world_image_version: WORLD_IMAGE_VERSION,
    };
    
    let entry = json!({
        "ts": chrono::Utc::now().to_rfc3339(),
        "event_type": "command_complete",
        "span_id": span_id,
        "exit": exit_code,
        "scopes_used": scopes,
        "fs_diff": diff,
        "replay_context": replay_context,
        // ... other fields
    });
    
    append_to_trace(&entry)?;
    Ok(())
}

// Replay with context reconstruction
pub fn replay_span(span_id: &str) -> Result<ReplayResult> {
    let span = load_span(span_id)?;
    let context = span.replay_context;
    
    // Warn on context drift
    if context.policy_commit != get_policy_git_hash()? {
        eprintln!("⚠️  Policy has changed since span was recorded");
    }
    
    if context.world_image_version != WORLD_IMAGE_VERSION {
        eprintln!("⚠️  World image version differs from original");
    }
    
    // Reconstruct environment
    let mut env = HashMap::new();
    if let Some(path) = context.path {
        env.insert("PATH".into(), path);
    }
    
    // Execute in fresh world with same context
    let world = SessionWorld::ephemeral()?;
    world.set_umask(context.umask)?;
    world.execute_with_env(&span.cmd, &context.cwd, env)
}
```

**Accept:** `substrate replay <span>` reproduces output/exit inside a fresh world.

### 4. Local Agent API (Unix Socket)

Programmatic interface for AI agents.

**Implementation:**
```rust
// crates/api/src/main.rs
use axum::{
    extract::{Path, State, Json},
    response::Response,
    routing::{get, post},
    Router,
};
use tokio::net::UnixListener;
use tower::ServiceBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let socket_path = home_dir().join(".substrate/sock/agent.sock");
    std::fs::create_dir_all(socket_path.parent().unwrap())?;
    
    let app = Router::new()
        .route("/v1/execute", post(execute_command))
        .route("/v1/stream", post(stream_command))
        .route("/v1/trace/:span_id", get(get_trace))
        .route("/v1/request_scopes", post(request_scopes));
    
    let listener = UnixListener::bind(&socket_path)?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Deserialize)]
struct ExecuteRequest {
    profile: Option<String>,
    cmd: String,
    cwd: Option<String>,
    env: Option<HashMap<String, String>>,
    pty: bool,
    agent_id: String,  // REQUIRED for API callers
    budget: Option<Budget>,
}

#[derive(Deserialize)]
struct Budget {
    max_execs: Option<u32>,
    max_runtime_ms: Option<u64>,
    max_egress_bytes: Option<u64>,  // Network egress limit
}

#[derive(Clone)]
struct AgentBudgetTracker {
    agent_id: String,
    execs_remaining: AtomicU32,
    runtime_remaining_ms: AtomicU64,
    egress_remaining: AtomicU64,
}

#[derive(Serialize)]
struct ExecuteResponse {
    exit: i32,
    span_id: String,
    stdout_b64: String,
    stderr_b64: String,
    scopes_used: Vec<String>,
}

async fn execute_command(Json(req): Json<ExecuteRequest>) -> Result<Json<ExecuteResponse>> {
    // REQUIRE agent_id for all API calls
    if req.agent_id.is_empty() {
        return Err(anyhow!("agent_id is required for API calls"));
    }
    
    // Load profile/policy
    let policy = load_policy(req.profile.as_deref().unwrap_or("default"))?;
    
    // Set agent_id in environment
    let mut env = req.env.unwrap_or_default();
    env.insert("SUBSTRATE_AGENT_ID".into(), req.agent_id.clone());
    
    // Apply and track budget
    if let Some(budget) = req.budget {
        let tracker = AGENT_BUDGETS.entry(req.agent_id.clone())
            .or_insert_with(|| AgentBudgetTracker::new(&req.agent_id, budget));
        
        // Check budget before execution
        if !tracker.can_execute() {
            trace::budget_exceeded(&req.agent_id, "max_execs")?;
            return Err(anyhow!("Budget exceeded: max executions reached"));
        }
        
        tracker.decrement_exec();
    }
    
    // Pass scope tokens via sealed FD, not env strings
    let scope_fd = create_sealed_scope_token(&policy)?;
    env.insert("SUBSTRATE_SCOPE_FD".into(), scope_fd.to_string());
    
    // Execute in world
    let world = SessionWorld::ensure_started(WorldSpec::from_policy(&policy))?;
    let result = world.execute(&req.cmd, &req.cwd.unwrap_or_default(), env)?;
    
    // Scrub scope token after exec
    close(scope_fd)?;
    
    Ok(Json(ExecuteResponse {
        exit: result.exit_code,
        span_id: result.span_id,
        stdout_b64: base64::encode(result.stdout),
        stderr_b64: base64::encode(result.stderr),
        scopes_used: result.scopes_used,
    }))
}

// PTY streaming with proper byte handling
async fn handle_pty_stream(socket: WebSocket) -> Result<()> {
    use portable_pty::{PtySystem, CommandBuilder};
    
    let pty_system = portable_pty::native_pty_system();
    let pair = pty_system.openpty(PtySize::default())?;
    
    // Handle vim/fzf without corruption
    let mut reader = pair.master.try_clone_reader()?;
    let mut writer = pair.master.take_writer()?;
    
    // Bidirectional byte streaming
    tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(n) if n > 0 => {
                    socket.send(Message::Binary(buf[..n].to_vec())).await?;
                }
                _ => break,
            }
        }
    });
    
    Ok(())
}

async fn stream_command(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| handle_pty_stream(socket))
}
```

**Usage:**
```bash
# From AI agent
curl --unix-socket ~/.substrate/sock/agent.sock \
  -X POST http://localhost/v1/execute \
  -d '{"cmd": "npm test", "agent_id": "claude"}'
```

**Accept:** 
- Returns `{exit, span_id, stdout_b64, stderr_b64, scopes_used}`
- Rejects calls without `agent_id`
- Returns 429-like error on budget exhaustion
- PTY handles vim/fzf without byte corruption

### 5. Policy Hot-Reload + Per-Directory Profiles

Live policy updates and automatic profile selection.

**Implementation:**
```rust
// crates/broker/src/reload.rs
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn watch_policy(path: &Path) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1))?;
    watcher.watch(path, RecursiveMode::NonRecursive)?;
    
    std::thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(event) => {
                    if let Ok(new_policy) = load_policy_file(path) {
                        GLOBAL_BROKER.write().unwrap().policy = RwLock::new(new_policy);
                        eprintln!("✓ Policy reloaded");
                    }
                }
                Err(_) => break,
            }
        }
    });
    
    Ok(())
}

pub fn find_profile() -> String {
    let mut dir = std::env::current_dir().unwrap();
    
    loop {
        let profile_file = dir.join(".substrate-profile");
        if profile_file.exists() {
            if let Ok(profile) = std::fs::read_to_string(profile_file) {
                return profile.trim().to_string();
            }
        }
        
        if !dir.pop() {
            break;
        }
    }
    
    "default".to_string()
}
```

**Accept:** Editing policy affects current session immediately; `.substrate-profile` auto-selects policy.

## Follow-up Priorities (MVP-class)

### 6. FS-Diff v2 & Network Allowlist

**Overlayfs for isolated commands (with gotcha handling):**
```rust
pub fn execute_with_overlay(cmd: &str) -> Result<FsDiff> {
    // CRITICAL: upper/work must be on same filesystem
    let overlay_base = PathBuf::from("/var/lib/substrate/overlay");
    std::fs::create_dir_all(&overlay_base)?;
    
    let world_id = format!("ovl_{}", uuid::Uuid::now_v7());
    let overlay_dir = overlay_base.join(&world_id);
    
    let upper = overlay_dir.join("upper");
    let work = overlay_dir.join("work");
    let merged = overlay_dir.join("merged");
    
    std::fs::create_dir_all(&upper)?;
    std::fs::create_dir_all(&work)?;
    std::fs::create_dir_all(&merged)?;
    
    // Mount overlay
    mount_overlay(&project_dir, &upper, &work, &merged)?;
    
    // Execute command with merged as root
    let result = execute_in_dir(cmd, &merged)?;
    
    // Compute diff from upper directory (with size limits)
    let diff = compute_fs_diff_smart(&upper)?;
    
    // Cleanup
    umount2(&merged, MntFlags::MNT_DETACH)?;
    std::fs::remove_dir_all(&overlay_dir)?;
    
    Ok(diff)
}

fn compute_fs_diff_smart(upper: &Path) -> Result<FsDiff> {
    const MAX_TRACKED_DIRS: usize = 100;
    const MAX_FILE_LIST: usize = 1000;
    
    let mut diff = FsDiff::default();
    let mut dir_count = 0;
    
    // For huge installs, track top-level changes + hash
    for entry in WalkDir::new(upper).max_depth(3) {
        let entry = entry?;
        
        if entry.file_type().is_dir() {
            dir_count += 1;
            if dir_count <= MAX_TRACKED_DIRS {
                diff.created_dirs.push(entry.path().to_path_buf());
            }
        } else if diff.writes.len() < MAX_FILE_LIST {
            diff.writes.push(entry.path().to_path_buf());
        }
    }
    
    // If we hit limits, add summary + tree hash
    if dir_count > MAX_TRACKED_DIRS || diff.writes.len() >= MAX_FILE_LIST {
        diff.truncated = true;
        diff.tree_hash = Some(hash_directory_tree(upper)?);
        diff.summary = Some(format!(
            "{} dirs, {} files (truncated, see tree_hash)",
            dir_count, 
            count_files(upper)?
        ));
    }
    
    Ok(diff)
}
```

**nftables in network namespace:**
```bash
# Inside world's netns
nft add table inet substrate
nft add chain inet substrate output { type filter hook output priority 0; }
nft add rule inet substrate output ip daddr 10.0.0.0/8 accept
nft add rule inet substrate output tcp dport 443 ip daddr github.com accept
nft add rule inet substrate output drop
```

### 7. LD_PRELOAD for Telemetry Only (Safe Injection)

Load intercept library for visibility, not enforcement - ONLY inside worlds.

```c
// crates/preload/src/intercept.c
int execve(const char* pathname, char* const argv[], char* const envp[]) {
    // CRITICAL: Telemetry only, never enforce
    if (getenv("SUBSTRATE_PRELOAD") != NULL) {
        log_execution("execve", pathname, argv);
    }
    
    // Always allow - we're telemetry, not enforcement
    orig_execve_t orig = dlsym(RTLD_NEXT, "execve");
    return orig(pathname, argv, envp);
}
```

```rust
// Only inject inside worlds, never on host
impl SessionWorld {
    pub fn execute_with_preload(&self, cmd: &str, env: HashMap<String, String>) -> Result<Output> {
        let mut env = env;
        
        // Gate preload with explicit env var
        if self.spec.enable_preload {
            env.insert("SUBSTRATE_PRELOAD".into(), "1".into());
            env.insert(
                "LD_PRELOAD".into(), 
                "/usr/lib/substrate/intercept.so".into()
            );
        }
        
        // Clear LD_PRELOAD when leaving world
        let result = Command::new(cmd)
            .envs(&env)
            .output()?;
            
        Ok(result)
    }
}
```

### 8. HRM On-Ramp (Scaffolding)

Prepare for future ML features without impacting execution.

```rust
// crates/spec/src/lib.rs
pub fn parse_help(tool: &str) -> Result<ToolSpec> {
    let output = Command::new(tool).arg("--help").output()?;
    // Parse help text to structured spec
}

// crates/synth/src/lib.rs
pub fn generate_examples(spec: &ToolSpec) -> Result<Vec<Example>> {
    // Use LLM to generate NL→CLI pairs
}

pub fn verify_in_world(example: &Example) -> Result<bool> {
    let world = SessionWorld::ephemeral()?;
    let result = world.execute(&example.command)?;
    Ok(result.exit_code == 0)
}
```

### 9. Kuzu Graph Database (Privacy-Aware)

Track relationships between commands, files, and agents with opt-in indexing.

```rust
// crates/graph/src/lib.rs
use kuzu::{Connection, Database};

pub struct PrivacyConfig {
    ignore_paths: Vec<PathBuf>,
    hash_code_only: bool,
    index_user_docs: bool,  // Default: false
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            ignore_paths: vec![
                PathBuf::from("~/.ssh"),
                PathBuf::from("~/Library"),
                PathBuf::from("/var/log/auth*"),
                PathBuf::from("**/.env"),
                PathBuf::from("**/secrets"),
            ],
            hash_code_only: true,
            index_user_docs: false,
        }
    }
}

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute(r#"
        CREATE NODE TABLE Dir(path STRING, PRIMARY KEY(path))
        CREATE NODE TABLE File(path STRING, hash STRING, PRIMARY KEY(path))
        CREATE NODE TABLE Span(id STRING, cmd STRING, exit INT, PRIMARY KEY(id))
        CREATE NODE TABLE Agent(id STRING, type STRING, PRIMARY KEY(id))
        
        CREATE REL TABLE WROTE(FROM Span TO File, bytes INT)
        CREATE REL TABLE READ(FROM Span TO File)
        CREATE REL TABLE EXECUTED_BY(FROM Span TO Agent)
        CREATE REL TABLE PARENT_OF(FROM Span TO Span)
    "#)?;
    Ok(())
}

pub fn record_span_complete(span: &SpanData, privacy: &PrivacyConfig) -> Result<()> {
    let conn = connect()?;
    
    // Insert span node
    conn.execute("CREATE (:Span {id: $1, cmd: $2, exit: $3})",
        &[&span.id, &span.cmd, &span.exit])?;
    
    // Insert edges for file writes (respecting privacy)
    for file in &span.fs_diff.writes {
        if should_index_file(file, privacy)? {
            let hash = if should_hash_file(file, privacy) {
                Some(hash_file(file)?)
            } else {
                None  // Just track presence, not content
            };
            
            conn.execute(
                "MATCH (s:Span {id: $1}), (f:File {path: $2}) CREATE (s)-[:WROTE]->(f)",
                &[&span.id, file]
            )?;
        }
    }
    
    Ok(())
}

fn should_index_file(path: &Path, privacy: &PrivacyConfig) -> bool {
    // Skip ignored paths
    for ignored in &privacy.ignore_paths {
        if path.starts_with(ignored) {
            return false;
        }
    }
    
    // Only index code by default
    if !privacy.index_user_docs {
        let ext = path.extension().and_then(|s| s.to_str());
        let code_exts = ["rs", "js", "ts", "py", "go", "c", "cpp", "java"];
        return ext.map_or(false, |e| code_exts.contains(&e));
    }
    
    true
}

// CLI queries
pub fn what_changed(span_id: &str) -> Result<Vec<String>> {
    let conn = connect()?;
    conn.query("MATCH (s:Span {id: $1})-[:WROTE]->(f:File) RETURN f.path", &[span_id])
}
```

### 10. Test Suites

**Golden span tests:**
```rust
#[test]
fn test_replay_npm_install() {
    let golden = load_golden_span("npm_install.json");
    let result = substrate_replay(&golden.span_id)?;
    assert_eq!(result.exit_code, golden.exit_code);
    assert_eq!(result.fs_diff, golden.fs_diff);
}
```

**Security tests:**
```rust
#[test]
fn test_deny_sudo() {
    let result = execute_with_policy("restricted", "sudo apt update");
    assert_eq!(result.exit_code, 126);
    assert!(result.stderr.contains("denied"));
}
```

## Policy Schema & Configuration

### JSON Schema for Policy Validation

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["id", "name"],
  "properties": {
    "id": {"type": "string", "pattern": "^[a-z0-9-]+$"},
    "name": {"type": "string"},
    "mode": {
      "type": "string",
      "enum": ["observe", "enforce"],
      "default": "observe"
    },
    "fs": {
      "type": "object",
      "properties": {
        "read": {"type": "array", "items": {"type": "string"}},
        "write": {"type": "array", "items": {"type": "string"}}
      }
    },
    "net": {
      "type": "object",
      "properties": {
        "allowed": {"type": "array", "items": {"type": "string"}},
        "egress_budget": {
          "type": "object",
          "properties": {
            "bytes_per_sec": {"type": "integer"},
            "total_bytes": {"type": "integer"}
          }
        }
      }
    },
    "commands": {
      "type": "object",
      "properties": {
        "allowed": {"type": "array", "items": {"type": "string"}},
        "denied": {"type": "array", "items": {"type": "string"}},
        "isolated": {"type": "array", "items": {"type": "string"}}
      }
    },
    "world": {
      "type": "object",
      "properties": {
        "reuse_session": {"type": "boolean", "default": true},
        "enable_preload": {"type": "boolean", "default": false},
        "isolate_network": {"type": "boolean", "default": true},
        "limits": {
          "type": "object",
          "properties": {
            "cpu": {"type": "string", "pattern": "^[0-9]+(\\.[0-9]+)?$"},
            "memory": {"type": "string", "pattern": "^[0-9]+(Ki|Mi|Gi)$"}
          }
        }
      }
    },
    "approval": {
      "type": "object",
      "properties": {
        "interactive": {"type": "boolean", "default": true},
        "auto_approve": {"type": "array", "items": {"type": "string"}}
      }
    }
  }
}
```

### Minimal Policy Example (Production-Ready)

```yaml
# ~/.substrate/policies/default.yaml
id: default
name: Development Policy
mode: observe  # Start in observe mode, switch to enforce when ready

fs:
  read: ["/**"]  # Read anything
  write:
    - "$PROJECT/**"  # Write only in project
    - "/tmp/**"
    - "$HOME/.cache/**"
    - "$HOME/.npm/**"
    - "$HOME/.cargo/**"

net:
  allowed:
    - github.com
    - registry.npmjs.org
    - pypi.org
    - crates.io
    - pkg.go.dev
  dns:
    pinned: true           # Pin to 127.0.0.53 stub resolver
    refresh_secs: 60       # Refresh IP sets every minute
  egress_budget:
    bytes_per_sec: 10485760  # 10 MB/s
    total_bytes: 1073741824  # 1 GB total per session

commands:
  denied:
    - "sudo *"
    - "rm -rf /"
    - "chmod 777 *"
    - "curl * | sh"  # Prevent curl | bash patterns
  isolated:  # Run these in ephemeral overlayfs
    - "pip install *"
    - "npm install *"
    - "cargo install *"
    - "go get *"
    - "gem install *"

world:
  reuse_session: true
  isolate_network: true
  enable_preload: false  # Enable when ready for telemetry
  limits:
    cpu: "2"      # 2 CPUs
    memory: "2Gi" # 2 GB RAM

approval:
  interactive: true
  auto_approve:
    - "git *"
    - "cargo build"
    - "cargo test"
    - "npm test"
    - "make"

# Privacy settings for graph database
privacy:
  ignore_paths:
    - "~/.ssh"
    - "~/Library"
    - "**/.env"
    - "**/secrets"
  hash_code_only: true
  index_user_docs: false
```

### Validation at Load Time

```rust
use jsonschema::JSONSchema;
use serde_yaml;

pub fn load_and_validate_policy(path: &Path) -> Result<Policy> {
    let yaml_str = std::fs::read_to_string(path)?;
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_str)?;
    
    // Convert YAML to JSON for schema validation
    let json_value = serde_json::to_value(yaml_value)?;
    
    // Load and compile schema
    let schema_str = include_str!("../schemas/policy.json");
    let schema = serde_json::from_str(schema_str)?;
    let compiled = JSONSchema::compile(&schema)?;
    
    // Validate
    let result = compiled.validate(&json_value);
    if let Err(errors) = result {
        for error in errors {
            eprintln!("Policy validation error: {}", error);
        }
        return Err(anyhow!("Policy validation failed"));
    }
    
    // Parse into struct
    let policy: Policy = serde_json::from_value(json_value)?;
    Ok(policy)
}
```

## Implementation Issues (Ready to Create)

### Backend Abstraction PRs (New - Ship First)
- **PR#1a**: `crates/world-api` - WorldBackend trait + structs, wire broker/trace/API to use it
- **PR#1b**: `crates/world-mac-lima` - MacLima backend + host runner (VM lifecycle, socket/SSH forwarding)
- **PR#1c**: `crates/world-agent` - Guest Linux service + systemd unit; reuses Linux isolation code
- **PR#1d**: Docs - `~/.lima/substrate.yaml` sample, brew install notes, backend matrix table

### Core Security PRs (Original)
- **PR#2**: Session World (LinuxLocal impl) - Linux bind mounts, cgroups v2, nftables, capabilities
- **PR#3**: Broker integration - evaluate() in shell, quick_check() in shim  
- **PR#4**: Span helpers + extended JSONL schema
- **PR#5**: Agent API - Unix socket with execute/stream/trace endpoints
- **PR#6**: Policy hot-reload + .substrate-profile detection
- **PR#7**: Overlayfs diffs + nftables network filtering
- **PR#8**: LD_PRELOAD telemetry library (Linux worlds only)
- **PR#9**: HRM scaffolding - spec parser, example generator
- **PR#10**: Kuzu graph database integration (privacy-aware)
- **PR#11**: Test suites - golden spans, security tests, platform parity, perf benchmarks

## Migration Path

1. **Opt-in via environment**: `SUBSTRATE_WORLD=enabled`
2. **Start with permissive policy**: Log violations without blocking
3. **Gradually tighten**: Move from observe to enforce
4. **Keep escape hatch**: `SHIM_BYPASS=1` for emergencies

## Success Metrics

- Policy evaluation < 10ms (measured with hyperfine)
- Zero breaking changes to existing Phase 1-3 functionality
- Agent API latency < 50ms for execute requests
- Session world reuse > 90% (not recreating each command)
- Graph queries < 100ms for typical spans

## Platform-Specific Considerations

| Platform        | Backend          | Isolation Stack                                  | Egress Control                       | Phase | Notes                    |
|-----------------|------------------|--------------------------------------------------|--------------------------------------|-------|--------------------------|
| **Linux**       | LinuxLocal       | namespaces + cgroups v2 + seccomp + overlayfs   | nftables                            | 4     | Default, native perf     |
| **Linux (opt)** | LinuxFirecracker | MicroVM + jailed rootfs                         | nftables                            | 4     | High-risk ephemeral only |
| **macOS**       | MacLima          | Linux VM (VZ via Lima) + same Linux stack inside| nftables (inside VM)                | 4     | 50ms overhead target     |
| **Windows**     | WSL2             | Linux distro + same Linux stack inside          | Windows Firewall or iptables in WSL2| 5     | Deferred                 |

### Linux Backends

**LinuxLocal** (Default):
- Native performance with namespaces, cgroups v2, nftables
- Fallback to Docker/Incus when user namespaces disabled
- Full seccomp filtering and capability dropping

**LinuxFirecracker** (Optional, High-Risk Only):
```yaml
world:
  high_isolation:
    enabled: false    # true only on Linux hosts
    provider: firecracker
    microvm_memory: "1Gi"
    microvm_vcpus: 1
```
- Use ONLY for high-risk ephemeral steps on Linux
- Not cross-platform (no nested virt on macOS)
- Significant overhead, reserve for critical isolation

### macOS via Lima

- Requires [Lima](https://github.com/lima-vm/lima) installation:
  ```bash
  brew install lima
  limactl start ~/.lima/substrate.yaml   # auto-run by substrate on first use
  ```
- Single warm VM with virtiofs mounts
- All enforcement happens inside VM using Linux stack
- Target: ≤50ms steady-state overhead

### Windows (Phase 5 - Deferred)
- WSL2 distro as the world implementation
- ConPTY for terminal handling
- Reuse Linux isolation code inside WSL2

## CLI UX Additions

New commands for managing worlds and policies:

```bash
# World management
substrate world status      # Show backend (LinuxLocal/MacLima), world id, mounts, limits
substrate world warm        # Start VM/container and agent (macOS: starts Lima VM)
substrate world restart     # Bounce the world/agent
substrate world doctor      # Health checks, verify Lima/VM status on macOS

# Policy management  
substrate policy reload     # Explicit reload (in addition to file watcher)
substrate policy validate   # Check policy against JSON schema
substrate policy show       # Display current effective policy

# Debugging
substrate trace <span_id>   # Show detailed span info
substrate replay <span_id>  # Replay command in fresh world
```

## Concrete Acceptance Criteria

### Platform Parity
- [ ] Linux & macOS both pass: FS bounds, egress allowlist, approvals, replay, PTY streaming
- [ ] Same policy format works on both platforms
- [ ] Agent API identical behavior across platforms

### Session World (Linux)
- [ ] `mount --make-rprivate /` verified (no host mount leaks)
- [ ] `pip install` fails to write outside project directory
- [ ] `curl example.com` blocked; `curl pypi.org` allowed
- [ ] `ps aux` inside world cannot see host processes (PID namespace)
- [ ] Graceful degradation when cgroups v2 unavailable
- [ ] User namespace fallback to Incus/Docker when disabled

### macOS Lima Backend
- [ ] `substrate` cold-starts the Lima VM on first run; subsequent runs do NOT block
- [ ] VM is kept warm; steady-state added latency ≤ **50 ms** per exec
- [ ] `pip install` obeys write bounds (project-only) and egress allowlist
- [ ] Session writes land on host via virtiofs when not isolated
- [ ] Isolated writes do NOT touch host; fully captured in diff
- [ ] DNS pinned to VM stub; allowlist refreshed atomically via nftables sets
- [ ] PTY tools (`vim`, `fzf`) stream cleanly through the agent
- [ ] `substrate replay <span>` reproduces exit/diff (top-N summary + tree hash)

### Broker in Hot Path
- [ ] Builtins (`cd`, `export`) logged as `builtin_command` spans
- [ ] `rm -rf /` denied with `policy_id` and pattern that matched
- [ ] Approval cache honors `once/session/always` and resets on profile change
- [ ] Atomic policy reloads without race conditions
- [ ] Observe mode logs violations without blocking
- [ ] Approval prompts include diff preview ("writes 134 files")

### Agent-Trace Spans
- [ ] `substrate replay <span>` reproduces `exit` and top-N diff summary
- [ ] Span includes `policy_commit` & `world_image` for reproducibility
- [ ] Replay warns on context drift (policy/world changes)
- [ ] Egress budget tracked and enforced per span

### Agent API
- [ ] Calls without `agent_id` rejected with clear error
- [ ] Budget exhaustion returns 429-like error + `rate_limit` span
- [ ] PTY streaming handles `vim` and `fzf` without corrupting bytes
- [ ] Scope tokens passed via sealed FD, not environment strings
- [ ] Per-agent budget tracking with atomic decrements

### Policy Hot-Reload
- [ ] Edit policy → next command uses new rules (no shell restart)
- [ ] `.substrate-profile` in subdir overrides parent profile
- [ ] Policy validation against JSON schema at load time
- [ ] Approval cache cleared on profile switch

### World API
- [ ] `WorldBackend` trait implemented by `LinuxLocal` and `MacLima`
- [ ] Broker/Trace/API do not depend on backend choice
- [ ] Backend auto-selection based on platform

### Security & Isolation
- [ ] OverlayFS upper/work on same filesystem (not tmpfs)
- [ ] DNS pinned to stub resolver (127.0.0.53) inside world
- [ ] LD_PRELOAD only injected inside worlds, never on host
- [ ] Seccomp baseline logs dangerous syscalls
- [ ] Network allowlist refreshes IPs atomically via nftables sets

### Privacy & Performance
- [ ] Kuzu ignores ~/.ssh, ~/Library, auth logs by default
- [ ] File hashing opt-in for non-code files
- [ ] FS diffs truncate at 1000 files with tree hash
- [ ] Policy evaluation < 10ms (measured with hyperfine)
- [ ] Agent API latency < 50ms for execute requests

## What We DON'T Change

- Keep the custom shell for REPL/builtins
- Keep PATH shims for telemetry
- Keep existing JSONL format (just add fields)
- Keep SHIM_BYPASS escape hatch

The broker + world carry enforcement, not complex rewrites!

## Architecture Decision Records

**ADR-004: macOS World via Lima** — To achieve policy parity on macOS in Phase 4 without rewriting isolation, we run a single, warm Linux VM via Lima and a `world-agent` inside it. All enforcement (FS/net/cgroups/seccomp/overlayfs) happens in Linux, identical to our Linux backend. Firecracker remains Linux-only for high-isolation ephemeral jobs and is not used on macOS.

## What We Defer (But Leave Hooks For)

- Full seccomp policy tuning (log today, tighten later)
- Sophisticated domain/SNI egress (start with IP sets + resolver refresher)
- HRM training/serving (ship the dataset generator now)
- Full Windows support (WSL2 follows same pattern as macOS/Lima)
- Complex network filtering (start with IP allowlists)
- Firecracker on macOS (nested virt not supported)