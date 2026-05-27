# All dirs (anywhere) with a Cargo.toml
CRATES_ALL := $(shell find . -maxdepth 3 -type f -name Cargo.toml -exec dirname {} \; | sort -u)

# Keep only things under ./crates/
CRATES := $(filter ./crates/%,$(CRATES_ALL))

# Strip the leading ./ for nicer printing (optional)
CRATES := $(patsubst ./%,%,$(CRATES))

# Command to run in each crate (override like: make walk-crates CRATE_CMD='cargo test')
CRATE_CMD ?= tokei .

# Base log directory
LOG_ROOT := target/crate-logs

# Date folder: e.g. 12-1-25 (MM-D-YY, UTC)
DATE_DIR := $(shell date -u +%m-%-d-%y)

# Full log directory for this date
LOG_DIR := $(LOG_ROOT)/$(DATE_DIR)

# One UTC timestamp per run, e.g. 20251201T153045Z
RUN_TS := $(shell date -u +%Y%m%dT%H%M%SZ)

# Final combined log will also be in the date folder
FINAL_LOG := $(LOG_DIR)/__all-crates.$(RUN_TS).log

WIN_TARGET ?= x86_64-pc-windows-msvc
ENABLE_WIN_PREFLIGHT ?= 0
PREFLIGHT_WIN_RUN_TESTS ?= 1

.PHONY: tokei-all-crates
tokei-all-crates:
	@mkdir -p "$(LOG_DIR)"
	@echo "Date dir (UTC): $(DATE_DIR)"
	@echo "Run timestamp (UTC): $(RUN_TS)"
	@echo "Log dir: $(LOG_DIR)"
	@echo "CRATES = $(CRATES)"
	@set -e; \
	for d in $(CRATES); do \
	  crate=$$(basename "$$d"); \
	  cmd_tag=$$(printf '%s\n' "$(CRATE_CMD)" | tr ' /' '-_'); \
	  log="$(LOG_DIR)/$${crate}_$${cmd_tag}_$(RUN_TS).log"; \
	  echo "===== BEGIN $$d =====" | tee "$$log"; \
	  (cd "$$d" && $(CRATE_CMD)) 2>&1 | tee -a "$$log"; \
	  echo "===== END $$d =====" | tee -a "$$log"; \
	  echo "" >> "$$log"; \
	done; \
	cat "$(LOG_DIR)"/*_*$$(printf '%s\n' "$(RUN_TS)").log > "$(FINAL_LOG)"; \
	echo "Combined log written to: $(FINAL_LOG)"

.PHONY: flightcheck

flightcheck:
	@echo "##flightcheck -- must run from repo root"
	@echo "##flightcheck -- must run pass for *integ tasks to be considered green"
	cargo fmt --all && cargo clippy --workspace --all-targets && cargo clean && cargo check --workspace --all-targets && cargo test --workspace --all-targets

.PHONY: integ-checks
integ-checks:
	@echo "##integ-checks -- must run from repo root"
	@echo "##integ-checks -- integration gate without cargo clean"
	cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo check --workspace --all-targets && cargo test --workspace --all-targets

.PHONY: preflight
preflight: flightcheck

.PHONY: dev-bootstrap
dev-bootstrap:
	@echo "##dev-bootstrap -- host-aware developer bootstrap"
	@if [ "$${OS:-}" = "Windows_NT" ]; then \
	  if ! command -v pwsh >/dev/null 2>&1; then \
	    echo "ERROR: pwsh is required on Windows. Install PowerShell 7 or run scripts/windows/dev-bootstrap.ps1 directly."; \
	    exit 2; \
	  fi; \
	  ENABLE_WIN_PREFLIGHT="$(ENABLE_WIN_PREFLIGHT)" pwsh -File "./scripts/windows/dev-bootstrap.ps1"; \
	else \
	  ENABLE_WIN_PREFLIGHT="$(ENABLE_WIN_PREFLIGHT)" bash "./scripts/dev/bootstrap.sh"; \
	fi

.PHONY: preflight-win
preflight-win:
	@echo "##preflight-win -- Linux-host Windows MSVC parity preflight"
	@echo "##preflight-win -- supplements but does not replace real Windows CI"
	@if [ "$$(uname -s)" != "Linux" ]; then \
	  echo "ERROR: preflight-win currently supports Linux hosts only"; \
	  exit 2; \
	fi
	@if ! command -v rustup >/dev/null 2>&1; then \
	  echo "ERROR: rustup is required"; \
	  exit 2; \
	fi
	@if ! command -v cargo-xwin >/dev/null 2>&1; then \
	  echo "Installing cargo-xwin..."; \
	  cargo install --locked cargo-xwin; \
	fi
	@if ! command -v clang >/dev/null 2>&1; then \
	  echo "ERROR: clang is required for cargo-xwin. Run 'make dev-bootstrap ENABLE_WIN_PREFLIGHT=1' and rerun."; \
	  exit 2; \
	fi
	@if [ "$(PREFLIGHT_WIN_RUN_TESTS)" = "1" ] && ! command -v wine >/dev/null 2>&1; then \
	  echo "ERROR: wine is required to run Windows-target tests. Run 'make dev-bootstrap ENABLE_WIN_PREFLIGHT=1' or rerun with PREFLIGHT_WIN_RUN_TESTS=0."; \
	  exit 2; \
	fi
	rustup target add $(WIN_TARGET)
	cargo xwin build -p substrate --bin substrate --target $(WIN_TARGET)
	cargo xwin check --workspace --all-targets --target $(WIN_TARGET)
	cargo xwin clippy --workspace --all-targets --target $(WIN_TARGET) -- -D warnings
	@if [ "$(PREFLIGHT_WIN_RUN_TESTS)" = "1" ]; then \
	  cargo xwin test --workspace --all-targets --target $(WIN_TARGET); \
	else \
	  echo "Skipping cargo xwin test because PREFLIGHT_WIN_RUN_TESTS=$(PREFLIGHT_WIN_RUN_TESTS)"; \
	fi

.PHONY: pre-ci
pre-ci:
	@echo "##pre-ci -- runs CI checks not covered by preflight (plus CI-flag fmt/clippy)"
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo build --workspace
	cargo rustc -p substrate-telemetry --profile dist --crate-type=rlib,cdylib
	cargo doc --workspace --no-deps
	@sh_files="$$(git ls-files '*.sh')"; \
	if command -v shellcheck >/dev/null 2>&1; then \
	  if [ -n "$$sh_files" ]; then \
	    printf '%s\n' "$$sh_files" | xargs shellcheck -x -S warning; \
	  else \
	    echo "No shell scripts found for shellcheck"; \
	  fi; \
	else \
	  echo "shellcheck not installed; skipping Shell lint"; \
	fi
	cargo run --bin substrate -- --version

# =========================
# Planning-system automation
# =========================

# Canonical Project Management system scripts root
PM_SYSTEM_SCRIPTS := docs/project_management/system/scripts
FSE_SYSTEM_SCRIPTS := docs/project_management/system/fse/scripts

# Compatibility vars kept for retired planning automation entrypoints.
FEATURE_DIR ?=

OWNED_PATHS ?=

# Planning agent id for pm-run-planning-agent
AGENT ?=

# PWS id for pm-run-pws
PWS_ID ?=

# Pre-planning research orchestrator options
START_AT ?=
POLL_S ?= 60

ADR ?=

CODEX_PROFILE ?=
CODEX_MODEL ?=
CODEX_JSONL ?= 0
EMIT_JSON ?= 0
PROVING_RUN_FACTS ?=
PROVING_RUN_HUMAN_INPUTS ?=
PROVING_RUN_CLOSEOUT_OUTPUT ?= proving-run-closeout.json

.PHONY: pm-prepare-proving-run-closeout
pm-prepare-proving-run-closeout:
	@if [ -z "$(PROVING_RUN_FACTS)" ]; then echo "ERROR: set PROVING_RUN_FACTS=<path-to-lifecycle-facts.json>"; exit 2; fi
	@set -euo pipefail; \
	cmd="python3 docs/project_management/system/scripts/execution/prepare_proving_run_closeout.py --facts \"$(PROVING_RUN_FACTS)\" --output \"$(PROVING_RUN_CLOSEOUT_OUTPUT)\""; \
	if [ -n "$(PROVING_RUN_HUMAN_INPUTS)" ]; then cmd="$$cmd --human-inputs \"$(PROVING_RUN_HUMAN_INPUTS)\""; fi; \
	eval "$$cmd"

.PHONY: planning-validate planning-lint planning-micro-lint pm-pws-plan pm-run-pws pm-full-planning-orchestrate pm-pre-full-planning-converge pm-post-full-planning-converge pm-planning-pipeline pm-run-planning-agent pm-pre-planning-research pm-pre-planning-from-adr pm-fse-pre-planning-from-adr planning-lint-ps
planning-validate planning-lint planning-micro-lint pm-pws-plan pm-run-pws pm-full-planning-orchestrate pm-pre-full-planning-converge pm-post-full-planning-converge pm-planning-pipeline pm-run-planning-agent pm-pre-planning-research pm-pre-planning-from-adr pm-fse-pre-planning-from-adr planning-lint-ps:
	@echo "ERROR: target '$@' was retired with project-management pack automation. See docs/PROJECT_MANAGEMENT_RETIREMENT.md." >&2
	@exit 2

.PHONY: adr-check
adr-check:
	@if [ -z "$(ADR)" ]; then echo "ERROR: set ADR=docs/project_management/adrs/<bucket>/ADR-XXXX-....md"; exit 2; fi
	python3 $(PM_SYSTEM_SCRIPTS)/planning/check_adr_exec_summary.py --adr "$(ADR)"

.PHONY: adr-fix
adr-fix:
	@if [ -z "$(ADR)" ]; then echo "ERROR: set ADR=docs/project_management/adrs/<bucket>/ADR-XXXX-....md"; exit 2; fi
	python3 $(PM_SYSTEM_SCRIPTS)/planning/check_adr_exec_summary.py --adr "$(ADR)" --fix

.PHONY: pm-lift-intake
pm-lift-intake:
	@if [ -z "$(FILE)" ]; then echo "ERROR: set FILE=<path/to/intake_or_adr.md>"; exit 2; fi
	@set -euo pipefail; \
	if [ "$(EMIT_JSON)" = "1" ]; then \
	  python3 $(PM_SYSTEM_SCRIPTS)/planning/pm_lift.py from-intake --intake "$(FILE)" --emit-json; \
	else \
	  python3 $(PM_SYSTEM_SCRIPTS)/planning/pm_lift.py from-intake --intake "$(FILE)"; \
	fi

.PHONY: pm-lift-pack
pm-lift-pack:
	@echo "ERROR: target '$@' was retired with project-management pack automation. See docs/PROJECT_MANAGEMENT_RETIREMENT.md." >&2
	@exit 2

.PHONY: pm-lift-diff
pm-lift-diff:
	@if [ -z "$(BASE)" ] || [ -z "$(HEAD)" ]; then echo "ERROR: set BASE=<git_ref> HEAD=<git_ref> (range is BASE..HEAD)"; exit 2; fi
	@set -euo pipefail; \
	if [ "$(EMIT_JSON)" = "1" ]; then \
	  python3 $(PM_SYSTEM_SCRIPTS)/planning/pm_lift.py from-git-diff --git-range "$(BASE)..$(HEAD)" --emit-json; \
	else \
	  python3 $(PM_SYSTEM_SCRIPTS)/planning/pm_lift.py from-git-diff --git-range "$(BASE)..$(HEAD)"; \
	fi

.PHONY: pm-lift-strict
pm-lift-strict:
	@echo "ERROR: target '$@' was retired with project-management pack automation. See docs/PROJECT_MANAGEMENT_RETIREMENT.md." >&2
	@exit 2

# =========================
# Cross-platform smoke (CI)
# =========================

# CI dispatch defaults (override as needed)
CURRENT_REF := $(shell git branch --show-current 2>/dev/null)
CI_WORKFLOW ?= .github/workflows/ci-testing.yml
CI_WORKFLOW_REF ?= $(CURRENT_REF)
CI_REMOTE ?= origin
CI_CLEANUP ?= 1
CI_CHECKOUT_REF ?=
CI_MODE ?=
CI_RUNNER_KIND ?=

.PHONY: ci-testing
ci-testing:
	@if [ -z "$(CI_WORKFLOW_REF)" ]; then echo "ERROR: set CI_WORKFLOW_REF=<ref> (ref must not be main/testing; use the orchestration/task ref)"; exit 2; fi
	@set -euo pipefail; \
	args="--workflow \"$(CI_WORKFLOW)\" --workflow-ref \"$(CI_WORKFLOW_REF)\" --remote \"$(CI_REMOTE)\""; \
	if [ -n "$(CI_CHECKOUT_REF)" ]; then args="$$args --checkout-ref \"$(CI_CHECKOUT_REF)\""; fi; \
	if [ -n "$(CI_RUNNER_KIND)" ]; then args="$$args --runner-kind \"$(CI_RUNNER_KIND)\""; fi; \
	if [ -n "$(CI_MODE)" ]; then args="$$args --mode \"$(CI_MODE)\""; fi; \
	if [ "$(CI_CLEANUP)" = "1" ]; then args="$$args --cleanup"; fi; \
	eval "scripts/ci/dispatch_ci_testing.sh $$args"

.PHONY: ci-compile-parity
ci-compile-parity:
	@$(MAKE) ci-testing CI_WORKFLOW=.github/workflows/ci-testing.yml CI_MODE=compile-parity

.PHONY: installers-container-smoke
installers-container-smoke:
	@bash tests/installers/pkg_manager_container_smoke.sh

# Retired pack automation compatibility entrypoints.
PLATFORM ?= linux
RUNNER_KIND ?= self-hosted
MACOS_RUNNER_KIND ?=
RUN_WSL ?= 0
RUN_INTEG_CHECKS ?= 0
SMOKE_SLICE_ID ?=
SMOKE_CHECKOUT_REF ?=
WORKFLOW ?=
WORKFLOW_REF ?= $(CURRENT_REF)
REMOTE ?= origin
CLEANUP ?= 1

FEATURE ?=
PACK_BUCKET ?=
DECISION_HEAVY ?= 0
CROSS_PLATFORM ?= 0
WSL_REQUIRED ?= 0
WSL_SEPARATE ?= 0
AUTOMATION ?= 0
BEHAVIOR_PLATFORMS ?=
CI_PARITY_PLATFORMS ?=
SLICE_PREFIX ?=

TASK_ID ?=
TASK_PLATFORM ?=
SLICE_ID ?=
CODE_TASK_ID ?=
TEST_TASK_ID ?=
PLATFORMS ?=
SMOKE_RUN_ID ?=

LAUNCH_CODEX ?= 0
VERIFY_ONLY ?= 0
NO_COMMIT ?= 0
SMOKE ?= 0
REMOVE_WORKTREES ?= 0
PRUNE_LOCAL ?= 0
PRUNE_REMOTE ?=
FORCE ?= 0
DRY_RUN ?= 0

.PHONY: planning-archive
planning-archive:
	@if [ -z "$(SRC)" ]; then echo "ERROR: set SRC=docs/project_management/<bucket>/<name>"; exit 2; fi
	@set -euo pipefail; \
	cmd="python3 $(PM_SYSTEM_SCRIPTS)/planning/archive_project_management_dir.py --src \"$(SRC)\""; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	if [ "$(ALLOW_DIRTY)" = "1" ]; then cmd="$$cmd --allow-dirty"; fi; \
	eval "$$cmd"

.PHONY: feature-smoke feature-smoke-all feature-smoke-behavior feature-smoke-wsl planning-new-feature planning-new-feature-ps triad-code-checks triad-test-checks triad-task-start triad-task-start-pair triad-task-start-complete triad-orch-ensure triad-task-start-platform-fixes triad-task-start-platform-fixes-from-smoke triad-task-start-integ-final triad-mark-noop-platform-fixes-completed triad-task-finish triad-feature-cleanup
feature-smoke feature-smoke-all feature-smoke-behavior feature-smoke-wsl planning-new-feature planning-new-feature-ps triad-code-checks triad-test-checks triad-task-start triad-task-start-pair triad-task-start-complete triad-orch-ensure triad-task-start-platform-fixes triad-task-start-platform-fixes-from-smoke triad-task-start-integ-final triad-mark-noop-platform-fixes-completed triad-task-finish triad-feature-cleanup:
	@echo "ERROR: target '$@' was retired with project-management pack automation. See docs/PROJECT_MANAGEMENT_RETIREMENT.md." >&2
	@exit 2
