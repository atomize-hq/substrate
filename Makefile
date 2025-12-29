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
