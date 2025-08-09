#!/bin/bash
set -euo pipefail

# Create ~/.substrate_bashenv for non-interactive shell environments
BASHENV_FILE="$HOME/.substrate_bashenv"
SHIM_DIR="$HOME/.cmdshim_rust"

echo "Creating $BASHENV_FILE for non-interactive shell support"

cat > "$BASHENV_FILE" <<'EOF'
# Substrate command tracing - BASH_ENV setup
# This file is sourced by non-interactive bash shells

# Exit early if already in nested shim context to avoid self-interception
if [[ "${SHIM_ACTIVE:-0}" -gt 0 ]]; then
    return 0
fi

# Set up paths
export SHIM_ORIGINAL_PATH="${SHIM_ORIGINAL_PATH:-$PATH}"

# Build clean shimmed PATH
SHIM_DIR="$HOME/.cmdshim_rust"
if [[ -d "$SHIM_DIR" ]]; then
    # Dedupe PATH components
    IFS=':' read -r -a parts <<<"$SHIM_DIR:$SHIM_ORIGINAL_PATH"
    declare -A seen
    deduped=()
    for d in "${parts[@]}"; do
        [[ -z "$d" ]] && continue
        key="${d%/}"
        [[ -n "${seen[$key]:-}" ]] && continue
        seen[$key]=1
        deduped+=("$d")
    done
    PATH="$(IFS=:; echo "${deduped[*]}")"
    unset parts seen deduped key
    
    # Pin shimmed commands to prevent bash hash table from using system versions
    hash -r
    for cmd in git npm npx node pnpm bun python python3 pip pip3 jq curl wget tar unzip make go cargo deno docker kubectl rg fd bat; do
        [[ -x "$SHIM_DIR/$cmd" ]] && hash -p "$SHIM_DIR/$cmd" "$cmd"
    done
fi

# Optional: Enable built-in command tracing (only if explicitly requested)
if [[ "${TRACE_BUILTINS:-0}" == "1" ]]; then
    : "${TRACE_SHELL:=$HOME/.trace_shell.jsonl}"
    if [[ -n "${TRACE_SHELL:-}" ]]; then
        exec {__xtrace_fd}>>"$TRACE_SHELL" 2>/dev/null || true
        if [[ -n "${__xtrace_fd:-}" ]]; then
            BASH_XTRACEFD=$__xtrace_fd
            
            # Handle macOS bash 3.2 compatibility (lacks EPOCHREALTIME)
            if [[ -n "${BASH_VERSINFO:-}" && ${BASH_VERSINFO[0]} -ge 5 ]]; then
                PS4='+CMD ${EPOCHREALTIME} ${BASH_SOURCE##*/}:${LINENO}: '
            else
                # Fallback for older bash versions (macOS default)
                PS4='+CMD '$(date +%s)'.'$(printf '%03d' $(($(date +%N 2>/dev/null || echo 0)/1000000)))' ${BASH_SOURCE##*/}:${LINENO}: '
            fi
            
            set -o xtrace
        fi
    fi
fi
EOF

chmod 644 "$BASHENV_FILE"

echo "Created $BASHENV_FILE"
echo ""
echo "To use with command tracing:"
echo "  export BASH_ENV=\"$BASHENV_FILE\""
echo "  # Then run your target application"
echo ""
echo "To enable built-in command tracing:"
echo "  export TRACE_BUILTINS=1"
echo "  export BASH_ENV=\"$BASHENV_FILE\""