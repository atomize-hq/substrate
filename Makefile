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

# Canonical Project Management system scripts root
PM_SYSTEM_SCRIPTS := docs/project_management/system/scripts
FSE_SYSTEM_SCRIPTS := docs/project_management/system/fse/scripts

# Feature directory under docs/project_management/packs/<bucket>/<feature>
FEATURE_DIR ?=

# Space-separated list of pack-relative paths to scan (scoped lint for planning agents)
OWNED_PATHS ?=

# Planning agent id for pm-run-planning-agent
AGENT ?=

# PWS id for pm-run-pws
PWS_ID ?=

# Pre-planning research orchestrator options
START_AT ?=
POLL_S ?= 60

# ADR path under docs/project_management/adrs/<bucket>/...
ADR ?=

CODEX_PROFILE ?=
CODEX_MODEL ?=
CODEX_JSONL ?= 0
EMIT_JSON ?= 0

.PHONY: planning-validate
planning-validate:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	python3 $(PM_SYSTEM_SCRIPTS)/planning/validate_tasks_json.py --feature-dir "$(FEATURE_DIR)"

.PHONY: planning-lint
planning-lint:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	$(PM_SYSTEM_SCRIPTS)/planning/lint.sh --feature-dir "$(FEATURE_DIR)"

.PHONY: planning-micro-lint
planning-micro-lint:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@if [ -z "$(OWNED_PATHS)" ]; then echo "ERROR: set OWNED_PATHS=\"<pack-relative paths you edited>\""; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/planning/micro_lint.sh --feature-dir \"$(FEATURE_DIR)\""; \
	if [ -n "$(AGENT)" ]; then cmd="$$cmd --agent \"$(AGENT)\""; fi; \
	cmd="$$cmd -- $(OWNED_PATHS)"; \
	eval "$$cmd"

.PHONY: pm-pws-plan
pm-pws-plan:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@python3 $(PM_SYSTEM_SCRIPTS)/planning/pm_pws_plan.py --feature-dir "$(FEATURE_DIR)"

.PHONY: pm-run-pws
pm-run-pws:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@if [ -z "$(PWS_ID)" ]; then echo "ERROR: set PWS_ID=<PWS_ID>"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/planning/run_pws_agent.sh --feature-dir \"$(FEATURE_DIR)\" --pws-id \"$(PWS_ID)\""; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	eval "$$cmd"

.PHONY: pm-full-planning-orchestrate
pm-full-planning-orchestrate:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/planning/full_planning_orchestrate.sh --feature-dir \"$(FEATURE_DIR)\""; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	eval "$$cmd"

.PHONY: pm-pre-full-planning-converge
pm-pre-full-planning-converge:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/planning/pre_full_planning_converge.sh --feature-dir \"$(FEATURE_DIR)\""; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	eval "$$cmd"

.PHONY: pm-post-full-planning-converge
pm-post-full-planning-converge:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/planning/post_full_planning_converge.sh --feature-dir \"$(FEATURE_DIR)\""; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	eval "$$cmd"

.PHONY: pm-planning-pipeline
pm-planning-pipeline:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/planning/planning_pipeline_orchestrate.sh --feature-dir \"$(FEATURE_DIR)\""; \
	if [ -n "$(START_AT)" ]; then cmd="$$cmd --start-at \"$(START_AT)\""; fi; \
	if [ -n "$(POLL_S)" ]; then cmd="$$cmd --poll-s \"$(POLL_S)\""; fi; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	eval "$$cmd"

.PHONY: pm-run-planning-agent
pm-run-planning-agent:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@if [ -z "$(AGENT)" ]; then echo "ERROR: set AGENT=spec_manifest|impact_map|min_spec_draft|ci_checkpoint|workstream_triage"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/planning/run_planning_agent.sh --feature-dir \"$(FEATURE_DIR)\" --agent \"$(AGENT)\""; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	eval "$$cmd"

.PHONY: pm-pre-planning-research
pm-pre-planning-research:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/planning/pre_planning_research_orchestrate.sh --feature-dir \"$(FEATURE_DIR)\""; \
	if [ -n "$(START_AT)" ]; then cmd="$$cmd --start-at \"$(START_AT)\""; fi; \
	if [ -n "$(POLL_S)" ]; then cmd="$$cmd --poll-s \"$(POLL_S)\""; fi; \
	CODEX_PROFILE="$(CODEX_PROFILE)" CODEX_MODEL="$(CODEX_MODEL)" CODEX_JSONL="$(CODEX_JSONL)" eval "$$cmd"

.PHONY: pm-pre-planning-from-adr
pm-pre-planning-from-adr:
	@if [ -z "$(ADR)" ]; then echo "ERROR: set ADR=docs/project_management/adrs/<bucket>/ADR-XXXX-....md"; exit 2; fi
	@set -euo pipefail; \
	if [ -n "$$(git status --porcelain=v1)" ]; then echo "ERROR: orchestration checkout is dirty; commit or stash before running"; exit 2; fi; \
	bucket="$(BUCKET)"; \
	if [ -z "$$bucket" ]; then bucket="$${PM_DEFAULT_PACK_BUCKET:-}"; fi; \
	if [ -z "$$bucket" ]; then bucket="draft"; fi; \
	cmd="$(PM_SYSTEM_SCRIPTS)/planning/scaffold_pre_planning_pack.sh --adr \"$(ADR)\" --bucket \"$$bucket\""; \
	if [ -n "$(FEATURE)" ]; then cmd="$$cmd --feature \"$(FEATURE)\""; fi; \
	feature_dir="$$(eval "$$cmd")"; \
	if [ -z "$$feature_dir" ]; then echo "ERROR: scaffold_pre_planning_pack.sh returned empty feature dir"; exit 2; fi; \
	tasks_path="$$feature_dir/tasks.json"; \
	if [ -n "$$(git status --porcelain=v1 -- "$$tasks_path")" ]; then \
	  git add -- "$$tasks_path"; \
	  if ! git diff --cached --quiet; then git commit -m "docs: bootstrap pre-planning pack"; fi; \
	fi; \
	if [ "$(RUN_PIPELINE)" = "1" ]; then \
	  $(MAKE) pm-planning-pipeline FEATURE_DIR="$$feature_dir" START_AT="$(START_AT)" POLL_S="$(POLL_S)" CODEX_PROFILE="$(CODEX_PROFILE)" CODEX_MODEL="$(CODEX_MODEL)" CODEX_JSONL="$(CODEX_JSONL)"; \
	else \
	  $(MAKE) pm-pre-planning-research FEATURE_DIR="$$feature_dir" START_AT="$(START_AT)" POLL_S="$(POLL_S)" CODEX_PROFILE="$(CODEX_PROFILE)" CODEX_MODEL="$(CODEX_MODEL)" CODEX_JSONL="$(CODEX_JSONL)"; \
	fi

.PHONY: pm-fse-pre-planning-from-adr
pm-fse-pre-planning-from-adr:
	@if [ -z "$(ADR)" ]; then echo "ERROR: set ADR=docs/project_management/adrs/<bucket>/ADR-XXXX-....md"; exit 2; fi
	@set -euo pipefail; \
	if [ -n "$$(git status --porcelain=v1)" ]; then echo "ERROR: orchestration checkout is dirty; commit or stash before running"; exit 2; fi; \
	bucket="$(BUCKET)"; \
	if [ -z "$$bucket" ]; then bucket="$${PM_DEFAULT_PACK_BUCKET:-}"; fi; \
	if [ -z "$$bucket" ]; then bucket="draft"; fi; \
	cmd="$(FSE_SYSTEM_SCRIPTS)/planning/scaffold_pre_planning_pack.sh --adr \"$(ADR)\" --bucket \"$$bucket\""; \
	if [ -n "$(FEATURE)" ]; then cmd="$$cmd --feature \"$(FEATURE)\""; fi; \
	feature_dir="$$(eval "$$cmd")"; \
	if [ -z "$$feature_dir" ]; then echo "ERROR: scaffold_pre_planning_pack.sh returned empty feature dir"; exit 2; fi; \
	tasks_path="$$feature_dir/tasks.json"; \
	if [ -n "$$(git status --porcelain=v1 -- "$$tasks_path")" ]; then \
	  git add -- "$$tasks_path"; \
	  if ! git diff --cached --quiet; then git commit -m "docs: bootstrap fse pre-planning pack"; fi; \
	fi; \
	cmd="$(FSE_SYSTEM_SCRIPTS)/planning/pre_planning_research_orchestrate.sh --feature-dir \"$$feature_dir\""; \
	if [ -n "$(START_AT)" ]; then cmd="$$cmd --start-at \"$(START_AT)\""; fi; \
	if [ -n "$(POLL_S)" ]; then cmd="$$cmd --poll-s \"$(POLL_S)\""; fi; \
	CODEX_PROFILE="$(CODEX_PROFILE)" CODEX_MODEL="$(CODEX_MODEL)" CODEX_JSONL="$(CODEX_JSONL)" eval "$$cmd"

.PHONY: planning-lint-ps
planning-lint-ps:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@if ! command -v pwsh >/dev/null 2>&1; then echo "ERROR: pwsh not found on PATH"; exit 2; fi
	pwsh -File $(PM_SYSTEM_SCRIPTS)/planning/lint.ps1 -FeatureDir "$(FEATURE_DIR)"

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
	@if [ -z "$(PACK)" ]; then echo "ERROR: set PACK=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(PACK)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: PACK must be under docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@set -euo pipefail; \
	if [ "$(EMIT_JSON)" = "1" ]; then \
	  python3 $(PM_SYSTEM_SCRIPTS)/planning/pm_lift.py from-impact-map --feature-dir "$(PACK)" --emit-json; \
	else \
	  python3 $(PM_SYSTEM_SCRIPTS)/planning/pm_lift.py from-impact-map --feature-dir "$(PACK)"; \
	fi

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
	@if [ -n "$(FILE)" ] && [ -n "$(PACK)" ]; then echo "ERROR: set only one of FILE or PACK"; exit 2; fi
	@if [ -z "$(FILE)" ] && [ -z "$(PACK)" ]; then echo "ERROR: set FILE=<path/to/intake_or_adr.md> or PACK=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if [ -n "$(PACK)" ] && ! echo "$(PACK)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: PACK must be under docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@set -euo pipefail; \
	if [ -n "$(FILE)" ]; then \
	  PM_LIFT_STRICT=1 python3 $(PM_SYSTEM_SCRIPTS)/planning/pm_lift_strict_check.py --intake "$(FILE)"; \
	else \
	  PM_LIFT_STRICT=1 python3 $(PM_SYSTEM_SCRIPTS)/planning/pm_lift_strict_check.py --feature-dir "$(PACK)"; \
	fi

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

# Dispatch defaults (override as needed)
PLATFORM ?= linux
	RUNNER_KIND ?= self-hosted
	MACOS_RUNNER_KIND ?=
	RUN_WSL ?= 0
	RUN_INTEG_CHECKS ?= 0
	SMOKE_SLICE_ID ?=
	SMOKE_CHECKOUT_REF ?=
	WORKFLOW ?= .github/workflows/feature-smoke.yml
WORKFLOW_REF ?= $(CURRENT_REF)
REMOTE ?= origin
CLEANUP ?= 1

.PHONY: feature-smoke
feature-smoke:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@if [ -z "$(WORKFLOW_REF)" ]; then echo "ERROR: set WORKFLOW_REF=<ref> (ref must not be main/testing; use the orchestration/task ref)"; exit 2; fi
	@if [ "$(PLATFORM)" = "wsl" ] && [ "$(RUNNER_KIND)" != "self-hosted" ]; then echo "ERROR: PLATFORM=wsl requires RUNNER_KIND=self-hosted"; exit 2; fi
	@if [ "$(RUN_WSL)" = "1" ] && [ "$(RUNNER_KIND)" != "self-hosted" ]; then echo "ERROR: RUN_WSL=1 requires RUNNER_KIND=self-hosted"; exit 2; fi
	@set -euo pipefail; \
	args="--feature-dir \"$(FEATURE_DIR)\" --runner-kind $(RUNNER_KIND) --platform $(PLATFORM) --workflow \"$(WORKFLOW)\" --workflow-ref \"$(WORKFLOW_REF)\" --remote \"$(REMOTE)\""; \
	if [ -n "$(MACOS_RUNNER_KIND)" ]; then args="$$args --macos-runner-kind $(MACOS_RUNNER_KIND)"; fi; \
	if [ "$(RUN_WSL)" = "1" ]; then args="$$args --run-wsl"; fi; \
	if [ "$(RUN_INTEG_CHECKS)" = "1" ]; then args="$$args --run-integ-checks"; fi; \
	if [ -n "$(SMOKE_CHECKOUT_REF)" ]; then args="$$args --checkout-ref \"$(SMOKE_CHECKOUT_REF)\""; fi; \
	if [ -n "$(SMOKE_SLICE_ID)" ]; then args="$$args --smoke-slice-id \"$(SMOKE_SLICE_ID)\""; fi; \
	if [ "$(CLEANUP)" = "1" ]; then args="$$args --cleanup"; fi; \
	eval "scripts/ci/dispatch_feature_smoke.sh $$args"

.PHONY: feature-smoke-all
feature-smoke-all:
	@$(MAKE) feature-smoke PLATFORM=all

.PHONY: feature-smoke-behavior
feature-smoke-behavior:
	@$(MAKE) feature-smoke PLATFORM=behavior

.PHONY: feature-smoke-wsl
feature-smoke-wsl:
	@$(MAKE) feature-smoke PLATFORM=wsl RUN_WSL=0 RUNNER_KIND=self-hosted

# =========================
# Planning pack scaffolding
# =========================

# New feature directory name under docs/project_management/packs/<bucket>/<feature>
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

.PHONY: planning-new-feature
planning-new-feature:
	@if [ -z "$(FEATURE)" ]; then echo "ERROR: set FEATURE=<feature_dir_name>"; exit 2; fi
	@set -euo pipefail; \
	bucket="$(PACK_BUCKET)"; \
	if [ -z "$$bucket" ]; then bucket="$${PM_DEFAULT_PACK_BUCKET:-}"; fi; \
	if [ -z "$$bucket" ]; then bucket="active"; fi; \
	cmd="$(PM_SYSTEM_SCRIPTS)/planning/new_feature.sh --feature \"$(FEATURE)\" --bucket \"$$bucket\""; \
	if [ -n "$(SLICE_PREFIX)" ]; then cmd="$$cmd --slice-prefix \"$(SLICE_PREFIX)\""; fi; \
	if [ "$(DECISION_HEAVY)" = "1" ]; then cmd="$$cmd --decision-heavy"; fi; \
	if [ "$(CROSS_PLATFORM)" = "1" ]; then cmd="$$cmd --cross-platform"; fi; \
	if [ -n "$(BEHAVIOR_PLATFORMS)" ]; then cmd="$$cmd --behavior-platforms \"$(BEHAVIOR_PLATFORMS)\""; fi; \
	if [ -n "$(CI_PARITY_PLATFORMS)" ]; then cmd="$$cmd --ci-parity-platforms \"$(CI_PARITY_PLATFORMS)\""; fi; \
	if [ "$(WSL_REQUIRED)" = "1" ]; then cmd="$$cmd --wsl-required"; fi; \
	if [ "$(WSL_SEPARATE)" = "1" ]; then cmd="$$cmd --wsl-separate"; fi; \
	if [ "$(AUTOMATION)" = "1" ]; then cmd="$$cmd --automation"; fi; \
	eval "$$cmd"; \
	$(MAKE) planning-validate FEATURE_DIR="docs/project_management/packs/$$bucket/$(FEATURE)"

.PHONY: planning-new-feature-ps
planning-new-feature-ps:
	@if [ -z "$(FEATURE)" ]; then echo "ERROR: set FEATURE=<feature_dir_name>"; exit 2; fi
	@if ! command -v pwsh >/dev/null 2>&1; then echo "ERROR: pwsh not found on PATH"; exit 2; fi
	@set -euo pipefail; \
	bucket="$(PACK_BUCKET)"; \
	if [ -z "$$bucket" ]; then bucket="$${PM_DEFAULT_PACK_BUCKET:-}"; fi; \
	if [ -z "$$bucket" ]; then bucket="active"; fi; \
	cmd="pwsh -File $(PM_SYSTEM_SCRIPTS)/planning/new_feature.ps1 -Feature \"$(FEATURE)\" -Bucket \"$$bucket\""; \
	if [ -n "$(SLICE_PREFIX)" ]; then cmd="$$cmd -SlicePrefix \"$(SLICE_PREFIX)\""; fi; \
	if [ "$(DECISION_HEAVY)" = "1" ]; then cmd="$$cmd -DecisionHeavy"; fi; \
	if [ "$(CROSS_PLATFORM)" = "1" ]; then cmd="$$cmd -CrossPlatform"; fi; \
	if [ -n "$(BEHAVIOR_PLATFORMS)" ]; then cmd="$$cmd -BehaviorPlatforms \"$(BEHAVIOR_PLATFORMS)\""; fi; \
	if [ -n "$(CI_PARITY_PLATFORMS)" ]; then cmd="$$cmd -CiParityPlatforms \"$(CI_PARITY_PLATFORMS)\""; fi; \
	if [ "$(WSL_REQUIRED)" = "1" ]; then cmd="$$cmd -WslRequired"; fi; \
	if [ "$(WSL_SEPARATE)" = "1" ]; then cmd="$$cmd -WslSeparate"; fi; \
	if [ "$(AUTOMATION)" = "1" ]; then cmd="$$cmd -Automation"; fi; \
	eval "$$cmd"; \
	$(MAKE) planning-validate FEATURE_DIR="docs/project_management/packs/$$bucket/$(FEATURE)"

.PHONY: planning-archive
planning-archive:
	@if [ -z "$(SRC)" ]; then echo "ERROR: set SRC=docs/project_management/<bucket>/<name>"; exit 2; fi
	@set -euo pipefail; \
	cmd="python3 $(PM_SYSTEM_SCRIPTS)/planning/archive_project_management_dir.py --src \"$(SRC)\""; \
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
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@if [ -z "$(TASK_ID)" ]; then echo "ERROR: set TASK_ID=<task-id>"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/triad/task_start.sh --feature-dir \"$(FEATURE_DIR)\" --task-id \"$(TASK_ID)\""; \
	if [ "$(LAUNCH_CODEX)" = "1" ]; then cmd="$$cmd --launch-codex"; fi; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	if [ -n "$(TASK_PLATFORM)" ]; then cmd="$$cmd --platform \"$(TASK_PLATFORM)\""; fi; \
	if [ "$(FORCE)" = "1" ]; then cmd="$$cmd --force"; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-task-start-pair
triad-task-start-pair:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@if [ -z "$(SLICE_ID)" ] && ( [ -z "$(CODE_TASK_ID)" ] || [ -z "$(TEST_TASK_ID)" ] ); then \
		  echo "ERROR: set SLICE_ID=<slice> OR set CODE_TASK_ID=<id> TEST_TASK_ID=<id>"; exit 2; \
		fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/triad/task_start_pair.sh --feature-dir \"$(FEATURE_DIR)\""; \
	if [ -n "$(SLICE_ID)" ]; then cmd="$$cmd --slice-id \"$(SLICE_ID)\""; fi; \
	if [ -n "$(CODE_TASK_ID)" ]; then cmd="$$cmd --code-task-id \"$(CODE_TASK_ID)\""; fi; \
	if [ -n "$(TEST_TASK_ID)" ]; then cmd="$$cmd --test-task-id \"$(TEST_TASK_ID)\""; fi; \
	if [ "$(LAUNCH_CODEX)" = "1" ]; then cmd="$$cmd --launch-codex"; fi; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	if [ "$(FORCE)" = "1" ]; then cmd="$$cmd --force"; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-task-start-complete
triad-task-start-complete:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@if [ -z "$(SLICE_ID)" ]; then echo "ERROR: set SLICE_ID=<slice>"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/triad/task_start_complete.sh --feature-dir \"$(FEATURE_DIR)\" --slice-id \"$(SLICE_ID)\""; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-orch-ensure
triad-orch-ensure:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/triad/orch_ensure.sh --feature-dir \"$(FEATURE_DIR)\""; \
	if [ -n "$(FROM_BRANCH)" ]; then cmd="$$cmd --from-branch \"$(FROM_BRANCH)\""; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-task-start-platform-fixes
triad-task-start-platform-fixes:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@if [ -z "$(SLICE_ID)" ]; then echo "ERROR: set SLICE_ID=<slice>"; exit 2; fi
	@if [ -z "$(PLATFORMS)" ]; then echo "ERROR: set PLATFORMS=linux,macos,windows[,wsl]"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/triad/task_start_platform_fixes.sh --feature-dir \"$(FEATURE_DIR)\" --slice-id \"$(SLICE_ID)\""; \
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
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@if [ -z "$(SLICE_ID)" ]; then echo "ERROR: set SLICE_ID=<slice>"; exit 2; fi
	@if [ -z "$(SMOKE_RUN_ID)" ]; then echo "ERROR: set SMOKE_RUN_ID=<gh-run-id>"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/triad/task_start_platform_fixes.sh --feature-dir \"$(FEATURE_DIR)\" --slice-id \"$(SLICE_ID)\" --from-smoke-run \"$(SMOKE_RUN_ID)\""; \
	if [ "$(LAUNCH_CODEX)" = "1" ]; then cmd="$$cmd --launch-codex"; fi; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-task-start-integ-final
triad-task-start-integ-final:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@if [ -z "$(SLICE_ID)" ]; then echo "ERROR: set SLICE_ID=<slice>"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/triad/task_start_integ_final.sh --feature-dir \"$(FEATURE_DIR)\" --slice-id \"$(SLICE_ID)\""; \
	if [ "$(LAUNCH_CODEX)" = "1" ]; then cmd="$$cmd --launch-codex"; fi; \
	if [ -n "$(CODEX_PROFILE)" ]; then cmd="$$cmd --codex-profile \"$(CODEX_PROFILE)\""; fi; \
	if [ -n "$(CODEX_MODEL)" ]; then cmd="$$cmd --codex-model \"$(CODEX_MODEL)\""; fi; \
	if [ "$(CODEX_JSONL)" = "1" ]; then cmd="$$cmd --codex-jsonl"; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-mark-noop-platform-fixes-completed
triad-mark-noop-platform-fixes-completed:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@if [ -z "$(SLICE_ID)" ]; then echo "ERROR: set SLICE_ID=<slice>"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/triad/mark_noop_platform_fixes_completed.sh --feature-dir \"$(FEATURE_DIR)\" --slice-id \"$(SLICE_ID)\""; \
	if [ -n "$(SMOKE_RUN_ID)" ]; then cmd="$$cmd --from-smoke-run \"$(SMOKE_RUN_ID)\""; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-task-finish
triad-task-finish:
	@if [ -z "$(TASK_ID)" ]; then echo "ERROR: set TASK_ID=<task-id>"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/triad/task_finish.sh --task-id \"$(TASK_ID)\""; \
	if [ "$(VERIFY_ONLY)" = "1" ]; then cmd="$$cmd --verify-only"; fi; \
	if [ "$(NO_COMMIT)" = "1" ]; then cmd="$$cmd --no-commit"; fi; \
	if [ "$(SMOKE)" = "1" ]; then cmd="$$cmd --smoke"; fi; \
	if [ -n "$(TASK_PLATFORM)" ]; then cmd="$$cmd --platform \"$(TASK_PLATFORM)\""; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"

.PHONY: triad-feature-cleanup
triad-feature-cleanup:
	@if [ -z "$(FEATURE_DIR)" ]; then echo "ERROR: set FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>"; exit 2; fi
	@if ! echo "$(FEATURE_DIR)" | grep -q '^docs/project_management/packs/'; then echo "ERROR: FEATURE_DIR must be under docs/project_management/packs/<bucket>/<feature> (legacy next/ is removed)"; exit 2; fi
	@set -euo pipefail; \
	cmd="$(PM_SYSTEM_SCRIPTS)/triad/feature_cleanup.sh --feature-dir \"$(FEATURE_DIR)\""; \
	if [ "$(REMOVE_WORKTREES)" = "1" ]; then cmd="$$cmd --remove-worktrees"; fi; \
	if [ "$(PRUNE_LOCAL)" = "1" ]; then cmd="$$cmd --prune-local-branches"; fi; \
	if [ -n "$(PRUNE_REMOTE)" ]; then cmd="$$cmd --prune-remote-branches \"$(PRUNE_REMOTE)\""; fi; \
	if [ "$(FORCE)" = "1" ]; then cmd="$$cmd --force"; fi; \
	if [ "$(DRY_RUN)" = "1" ]; then cmd="$$cmd --dry-run"; fi; \
	eval "$$cmd"
