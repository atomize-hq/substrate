# Netfilter Enforcement Verification

Use this playbook to verify the opt-in world netfilter contract on Linux and macOS. It is the
operator-facing companion to the three-way gate published in
[`world.net.filter`](../reference/config/world.md):

- `world.net.filter` decides whether the host may request enforcement.
- policy `net_allowed` decides whether the posture is allow-all, deny-all, or a restrictive allowlist.
- `WORLD_NETFILTER_ENABLE=1` decides whether the world backend may honor a requested run.

When all three gates line up for a restrictive posture, the runtime must enforce or fail closed.

## Doctor fields to inspect

`substrate world doctor --json` reports the published netfilter status block at
`.world.netfilter_status`:

- `requested`: whether the last run asked the backend to enforce outbound filtering.
- `enabled`: whether the backend was both requested and allowed to enforce.
- `world_netfilter_enable_present`: whether the world-service service currently has `WORLD_NETFILTER_ENABLE=1`.
- `last_failure_reason`: the last actionable enforcement failure for a requested run, if any.

## Linux privileged verification

This is the concrete privileged proof for real nftables installation in an isolated netns.

### Prerequisites

- Linux host
- root access or a privileged shell
- `iproute2`
- `nftables`
- Rust toolchain

### Command

From the repository root, run this exact workspace test command in a privileged shell:

```bash
cargo test -p world -- --ignored --nocapture
```

If you are not already root, use a sudo wrapper that preserves your toolchain `PATH`, for example:

```bash
sudo -E env PATH="$PATH" cargo test -p world -- --ignored --nocapture
```

### Expected evidence

- Pass:
  - the ignored `test_nftables_rules` test runs instead of failing immediately on missing `WORLD_NETFILTER_ENABLE`
  - the command exits `0`
- Skip:
  - the test prints an explicit skip message because the host cannot create the isolated netns or lacks the required Linux tooling
- Failure:
  - the command exits non-zero
  - the output includes nftables install/remove errors, unexpected root/tooling failures, or a regression in the isolated-netns path

### What to capture for closeout

- the full `cargo test -p world -- --ignored --nocapture` output
- the host identity used to run the command (`id` output is sufficient)
- any skip reason if the host cannot provide the privileged surface

## macOS Lima conformance smoke

This smoke path verifies that the Lima guest environment, host gate, and policy posture line up with
the published contract.

### Step 1: warm Lima with the backend guard enabled

From the repository root:

```bash
SUBSTRATE_WORLD_NETFILTER_ENABLE=1 scripts/mac/lima-warm.sh
```

You can verify the guest systemd env without reprovisioning:

```bash
scripts/mac/lima-warm.sh --check-only
```

The check-only output should report that the guest systemd env includes
`WORLD_NETFILTER_ENABLE=1`.

### Step 2: run the posture-aware smoke

```bash
PATH="$(pwd)/target/debug:$PATH" scripts/mac/smoke.sh --netfilter-conformance --log-dir artifacts/mac/netfilter-smoke-$(date -u '+%Y%m%d-%H%M%S')
```

The script creates a temporary no-workspace config/policy fixture inside the chosen log directory so
it does not mutate repository or user configuration.

### Expected posture results

- Allow-all posture (`net_allowed=["*"]`):
  - the probe command succeeds
  - `.world.netfilter_status.requested == false`
  - `.world.netfilter_status.enabled == false`
  - `.world.netfilter_status.world_netfilter_enable_present == true`
- Deny-all posture (`net_allowed=[]`):
  - the probe command fails because DNS/egress is blocked
  - `.world.netfilter_status.requested == true`
  - `.world.netfilter_status.enabled == true`
  - `.world.netfilter_status.world_netfilter_enable_present == true`
  - `.world.netfilter_status.last_failure_reason == null`

### What to capture for closeout

- `allow-all-world-doctor.json`
- `deny-all-world-doctor.json`
- the corresponding `*-probe.stdout.log`, `*-probe.stderr.log`, and `*-probe.exit` files

### If the smoke fails

- inspect `.world.netfilter_status.last_failure_reason`
- if `world_netfilter_enable_present` is `false`, rerun the Lima warm step with `SUBSTRATE_WORLD_NETFILTER_ENABLE=1`
- if `requested` is not what you expected, re-check the host gate and policy posture against the three-way gate contract in
  [`world.md`](../reference/config/world.md)

## Optional manual restrictive allowlist walkthrough

The automated macOS smoke intentionally stops at allow-all and deny-all. Use this manual path when
you need operator evidence for a named allowlist such as `["example.com"]`.

1. Start from a successful Lima warm step with `SUBSTRATE_WORLD_NETFILTER_ENABLE=1`.
2. Create a temporary no-workspace `SUBSTRATE_HOME` with:
   - `world.net.filter: false` in `config.yaml`
   - `net_allowed: ["example.com"]` in `policy.yaml`
3. Run commands with `SUBSTRATE_OVERRIDE_WORLD_NET_FILTER=1`.
4. Verify:
   - `substrate --world -c 'getent hosts example.com'` succeeds
   - `substrate --world -c 'getent hosts github.com'` fails
   - `substrate world doctor --json` reports `requested=true`, `enabled=true`, `world_netfilter_enable_present=true`

Capture the doctor JSON and both command transcripts if you need restrictive-posture evidence in a PR
or closeout note.
