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
	cargo fmt && cargo clippy --workspace --all-targets && cargo clean && cargo check --workspace --all-targets && cargo test --workspace --all-targets

.PHONY: integ-checks
integ-checks:
	@echo "##integ-checks -- must run from repo root"
	@echo "##integ-checks -- integration gate without cargo clean"
	cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo check --workspace --all-targets && cargo test --workspace --all-targets

.PHONY: preflight
preflight: flightcheck

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

# Feature directory under docs/project_management/next/<feature>
FEATURE_DIR ?=

# ADR path under docs/project_management/next/...
ADR ?=

.PHONY: planning-validate
planning-validate:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/next/<feature>"; exit 2; fi
	python3 scripts/planning/validate_tasks_json.py --feature-dir "$(FEATURE_DIR)"

.PHONY: planning-lint
planning-lint:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/next/<feature>"; exit 2; fi
	scripts/planning/lint.sh --feature-dir "$(FEATURE_DIR)"

.PHONY: planning-lint-ps
planning-lint-ps:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/next/<feature>"; exit 2; fi
	@if ! command -v pwsh >/dev/null 2>&1; then echo "ERROR: pwsh not found on PATH"; exit 2; fi
	pwsh -File scripts/planning/lint.ps1 -FeatureDir "$(FEATURE_DIR)"

.PHONY: adr-check
adr-check:
	@if [ -z "$(ADR)" ]; then echo "ERROR: set ADR=docs/project_management/next/ADR-XXXX-....md"; exit 2; fi
	python3 scripts/planning/check_adr_exec_summary.py --adr "$(ADR)"

.PHONY: adr-fix
adr-fix:
	@if [ -z "$(ADR)" ]; then echo "ERROR: set ADR=docs/project_management/next/ADR-XXXX-....md"; exit 2; fi
	python3 scripts/planning/check_adr_exec_summary.py --adr "$(ADR)" --fix

# =========================
# Cross-platform smoke (CI)
# =========================

# CI dispatch defaults (override as needed)
CI_WORKFLOW ?= .github/workflows/ci-testing.yml
CI_WORKFLOW_REF ?= testing
CI_REMOTE ?= origin
CI_CLEANUP ?= 1
CI_CHECKOUT_REF ?=

.PHONY: ci-testing
ci-testing:
	@set -euo pipefail; \
	args="--workflow \"$(CI_WORKFLOW)\" --workflow-ref \"$(CI_WORKFLOW_REF)\" --remote \"$(CI_REMOTE)\""; \
	if [ -n "$(CI_CHECKOUT_REF)" ]; then args="$$args --checkout-ref \"$(CI_CHECKOUT_REF)\""; fi; \
	if [ "$(CI_CLEANUP)" = "1" ]; then args="$$args --cleanup"; fi; \
	eval "scripts/ci/dispatch_ci_testing.sh $$args"

.PHONY: ci-compile-parity
ci-compile-parity:
	@$(MAKE) ci-testing CI_WORKFLOW=.github/workflows/ci-testing.yml

# Dispatch defaults (override as needed)
PLATFORM ?= linux
RUNNER_KIND ?= self-hosted
RUN_WSL ?= 0
RUN_INTEG_CHECKS ?= 0
WORKFLOW ?= .github/workflows/feature-smoke.yml
WORKFLOW_REF ?= feat/policy_and_config
REMOTE ?= origin
CLEANUP ?= 1

.PHONY: feature-smoke
feature-smoke:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/next/<feature>"; exit 2; fi
	@if [ "$(PLATFORM)" = "wsl" ] && [ "$(RUNNER_KIND)" != "self-hosted" ]; then echo "ERROR: PLATFORM=wsl requires RUNNER_KIND=self-hosted"; exit 2; fi
	@if [ "$(RUN_WSL)" = "1" ] && [ "$(RUNNER_KIND)" != "self-hosted" ]; then echo "ERROR: RUN_WSL=1 requires RUNNER_KIND=self-hosted"; exit 2; fi
	@set -euo pipefail; \
	args="--feature-dir \"$(FEATURE_DIR)\" --runner-kind $(RUNNER_KIND) --platform $(PLATFORM) --workflow \"$(WORKFLOW)\" --workflow-ref \"$(WORKFLOW_REF)\" --remote \"$(REMOTE)\""; \
	if [ "$(RUN_WSL)" = "1" ]; then args="$$args --run-wsl"; fi; \
	if [ "$(RUN_INTEG_CHECKS)" = "1" ]; then args="$$args --run-integ-checks"; fi; \
	if [ "$(CLEANUP)" = "1" ]; then args="$$args --cleanup"; fi; \
	eval "scripts/ci/dispatch_feature_smoke.sh $$args"

.PHONY: feature-smoke-all
feature-smoke-all:
	@$(MAKE) feature-smoke PLATFORM=all

.PHONY: feature-smoke-wsl
feature-smoke-wsl:
	@$(MAKE) feature-smoke PLATFORM=wsl RUN_WSL=0 RUNNER_KIND=self-hosted

# =========================
# Planning pack scaffolding
# =========================

# New feature directory name under docs/project_management/next/<feature>
FEATURE ?=
DECISION_HEAVY ?= 0
CROSS_PLATFORM ?= 0
WSL_REQUIRED ?= 0
WSL_SEPARATE ?= 0
AUTOMATION ?= 0
BEHAVIOR_PLATFORMS ?=
CI_PARITY_PLATFORMS ?=

.PHONY: planning-new-feature
planning-new-feature:
	@if [ -z "$(FEATURE)" ]; then echo "ERROR: set FEATURE=<feature_dir_name>"; exit 2; fi
	@set -euo pipefail; \
	cmd="scripts/planning/new_feature.sh --feature \"$(FEATURE)\""; \
	if [ "$(DECISION_HEAVY)" = "1" ]; then cmd="$$cmd --decision-heavy"; fi; \
	if [ "$(CROSS_PLATFORM)" = "1" ]; then cmd="$$cmd --cross-platform"; fi; \
	if [ -n "$(BEHAVIOR_PLATFORMS)" ]; then cmd="$$cmd --behavior-platforms \"$(BEHAVIOR_PLATFORMS)\""; fi; \
	if [ -n "$(CI_PARITY_PLATFORMS)" ]; then cmd="$$cmd --ci-parity-platforms \"$(CI_PARITY_PLATFORMS)\""; fi; \
	if [ "$(WSL_REQUIRED)" = "1" ]; then cmd="$$cmd --wsl-required"; fi; \
	if [ "$(WSL_SEPARATE)" = "1" ]; then cmd="$$cmd --wsl-separate"; fi; \
	if [ "$(AUTOMATION)" = "1" ]; then cmd="$$cmd --automation"; fi; \
	eval "$$cmd"; \
	$(MAKE) planning-validate FEATURE_DIR="docs/project_management/next/$(FEATURE)"

.PHONY: planning-new-feature-ps
planning-new-feature-ps:
	@if [ -z "$(FEATURE)" ]; then echo "ERROR: set FEATURE=<feature_dir_name>"; exit 2; fi
	@if ! command -v pwsh >/dev/null 2>&1; then echo "ERROR: pwsh not found on PATH"; exit 2; fi
	@set -euo pipefail; \
	cmd="pwsh -File scripts/planning/new_feature.ps1 -Feature \"$(FEATURE)\""; \
	if [ "$(DECISION_HEAVY)" = "1" ]; then cmd="$$cmd -DecisionHeavy"; fi; \
	if [ "$(CROSS_PLATFORM)" = "1" ]; then cmd="$$cmd -CrossPlatform"; fi; \
	if [ -n "$(BEHAVIOR_PLATFORMS)" ]; then cmd="$$cmd -BehaviorPlatforms \"$(BEHAVIOR_PLATFORMS)\""; fi; \
	if [ -n "$(CI_PARITY_PLATFORMS)" ]; then cmd="$$cmd -CiParityPlatforms \"$(CI_PARITY_PLATFORMS)\""; fi; \
	if [ "$(WSL_REQUIRED)" = "1" ]; then cmd="$$cmd -WslRequired"; fi; \
	if [ "$(WSL_SEPARATE)" = "1" ]; then cmd="$$cmd -WslSeparate"; fi; \
	if [ "$(AUTOMATION)" = "1" ]; then cmd="$$cmd -Automation"; fi; \
	eval "$$cmd"; \
	$(MAKE) planning-validate FEATURE_DIR="docs/project_management/next/$(FEATURE)"

.PHONY: planning-archive
planning-archive:
	@if [ -z "$(SRC)" ]; then echo "ERROR: set SRC=docs/project_management/<bucket>/<name>"; exit 2; fi
	@set -euo pipefail; \
	cmd="python3 scripts/planning/archive_project_management_dir.py --src \"$(SRC)\""; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	if [ "$(ALLOW_DIRTY)" = "1" ]; then cmd="$$cmd --allow-dirty"; fi; \
	eval "$$cmd"

# =========================
# Triad execution automation
# =========================

TASK_ID ?=
TASK_PLATFORM ?=
SLICE_ID ?=
CODE_TASK_ID ?=
TEST_TASK_ID ?=
PLATFORMS ?=
SMOKE_RUN_ID ?=

LAUNCH_CODEX ?= 0
CODEX_PROFILE ?=
CODEX_MODEL ?=
CODEX_JSONL ?= 0

VERIFY_ONLY ?= 0
NO_COMMIT ?= 0
SMOKE ?= 0

REMOVE_WORKTREES ?= 0
PRUNE_LOCAL ?= 0
PRUNE_REMOTE ?=
FORCE ?= 0
DRY_RUN ?= 0

.PHONY: triad-code-checks
triad-code-checks:
	cargo fmt
	cargo clippy --workspace --all-targets -- -D warnings

.PHONY: triad-test-checks
triad-test-checks:
	cargo fmt

.PHONY: triad-task-start
triad-task-start:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/next/<feature>"; exit 2; fi
	@if [ -z "$(TASK_ID)" ]; then echo "ERROR: set TASK_ID=<task-id>"; exit 2; fi
	@set -euo pipefail; \
	cmd="scripts/triad/task_start.sh --feature-dir \"$(FEATURE_DIR)\" --task-id \"$(TASK_ID)\""; \
	if [ "$(LAUNCH_CODEX)" = "1" ]; then cmd="$$cmd --launch-codex"; fi; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	if [ -n "$(TASK_PLATFORM)" ]; then cmd="$$cmd --platform \"$(TASK_PLATFORM)\""; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-task-start-pair
triad-task-start-pair:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/next/<feature>"; exit 2; fi
	@if [ -z "$(SLICE_ID)" ] && ( [ -z "$(CODE_TASK_ID)" ] || [ -z "$(TEST_TASK_ID)" ] ); then \
	  echo "ERROR: set SLICE_ID=<slice> OR set CODE_TASK_ID=<id> TEST_TASK_ID=<id>"; exit 2; \
	fi
	@set -euo pipefail; \
	cmd="scripts/triad/task_start_pair.sh --feature-dir \"$(FEATURE_DIR)\""; \
	if [ -n "$(SLICE_ID)" ]; then cmd="$$cmd --slice-id \"$(SLICE_ID)\""; fi; \
	if [ -n "$(CODE_TASK_ID)" ]; then cmd="$$cmd --code-task-id \"$(CODE_TASK_ID)\""; fi; \
	if [ -n "$(TEST_TASK_ID)" ]; then cmd="$$cmd --test-task-id \"$(TEST_TASK_ID)\""; fi; \
	if [ "$(LAUNCH_CODEX)" = "1" ]; then cmd="$$cmd --launch-codex"; fi; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-orch-ensure
triad-orch-ensure:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/next/<feature>"; exit 2; fi
	@set -euo pipefail; \
	cmd="scripts/triad/orch_ensure.sh --feature-dir \"$(FEATURE_DIR)\""; \
	if [ -n "$(FROM_BRANCH)" ]; then cmd="$$cmd --from-branch \"$(FROM_BRANCH)\""; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-task-start-platform-fixes
triad-task-start-platform-fixes:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/next/<feature>"; exit 2; fi
	@if [ -z "$(SLICE_ID)" ]; then echo "ERROR: set SLICE_ID=<slice>"; exit 2; fi
	@if [ -z "$(PLATFORMS)" ]; then echo "ERROR: set PLATFORMS=linux,macos,windows[,wsl]"; exit 2; fi
	@set -euo pipefail; \
	cmd="scripts/triad/task_start_platform_fixes.sh --feature-dir \"$(FEATURE_DIR)\" --slice-id \"$(SLICE_ID)\""; \
	IFS=',' read -r -a platforms <<<"$(PLATFORMS)"; \
	for p in "$${platforms[@]}"; do cmd="$$cmd --platform \"$$p\""; done; \
	if [ "$(LAUNCH_CODEX)" = "1" ]; then cmd="$$cmd --launch-codex"; fi; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-task-start-platform-fixes-from-smoke
triad-task-start-platform-fixes-from-smoke:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/next/<feature>"; exit 2; fi
	@if [ -z "$(SLICE_ID)" ]; then echo "ERROR: set SLICE_ID=<slice>"; exit 2; fi
	@if [ -z "$(SMOKE_RUN_ID)" ]; then echo "ERROR: set SMOKE_RUN_ID=<gh-run-id>"; exit 2; fi
	@set -euo pipefail; \
	cmd="scripts/triad/task_start_platform_fixes.sh --feature-dir \"$(FEATURE_DIR)\" --slice-id \"$(SLICE_ID)\" --from-smoke-run \"$(SMOKE_RUN_ID)\""; \
	if [ "$(LAUNCH_CODEX)" = "1" ]; then cmd="$$cmd --launch-codex"; fi; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-task-start-integ-final
triad-task-start-integ-final:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/next/<feature>"; exit 2; fi
	@if [ -z "$(SLICE_ID)" ]; then echo "ERROR: set SLICE_ID=<slice>"; exit 2; fi
	@set -euo pipefail; \
	cmd="scripts/triad/task_start_integ_final.sh --feature-dir \"$(FEATURE_DIR)\" --slice-id \"$(SLICE_ID)\""; \
	if [ "$(LAUNCH_CODEX)" = "1" ]; then cmd="$$cmd --launch-codex"; fi; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-task-finish
triad-task-finish:
	@if [ -z "$(TASK_ID)" ]; then echo "ERROR: set TASK_ID=<task-id>"; exit 2; fi
	@set -euo pipefail; \
	cmd="scripts/triad/task_finish.sh --task-id \"$(TASK_ID)\""; \
	if [ "$(VERIFY_ONLY)" = "1" ]; then cmd="$$cmd --verify-only"; fi; \
	if [ "$(NO_COMMIT)" = "1" ]; then cmd="$$cmd --no-commit"; fi; \
	if [ "$(SMOKE)" = "1" ]; then cmd="$$cmd --smoke"; fi; \
	if [ -n "$(TASK_PLATFORM)" ]; then cmd="$$cmd --platform \"$(TASK_PLATFORM)\""; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-feature-cleanup
triad-feature-cleanup:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/next/<feature>"; exit 2; fi
	@set -euo pipefail; \
	cmd="scripts/triad/feature_cleanup.sh --feature-dir \"$(FEATURE_DIR)\""; \
	if [ "$(REMOVE_WORKTREES)" = "1" ]; then cmd="$$cmd --remove-worktrees"; fi; \
	if [ "$(PRUNE_LOCAL)" = "1" ]; then cmd="$$cmd --prune-local-branches"; fi; \
	if [ -n "$(PRUNE_REMOTE)" ]; then cmd="$$cmd --prune-remote-branches \"$(PRUNE_REMOTE)\""; fi; \
	if [ "$(FORCE)" = "1" ]; then cmd="$$cmd --force"; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"
