#!/bin/bash
set -euo pipefail

# Stage substrate shims for command tracing
# This script deploys the built shim binary to intercept common development commands

SHIM_DIR="$HOME/.cmdshim_rust"
TARGET_BINARY="${1:-target/release/shim}"

# Commands to shim - curated list for development safety
# Based on proven working commands from manual testing
COMMANDS=(
    git npm npx node pnpm bun python python3 pip pip3
    jq curl wget tar unzip make go cargo deno docker kubectl
    rg fd bat
)

echo "🚀 Staging substrate shims in $SHIM_DIR"

# Validate target binary exists
if [[ ! -f "$TARGET_BINARY" ]]; then
    echo "❌ Error: Target binary not found at $TARGET_BINARY"
    echo "💡 Build the shim first: cargo build --release --bin shim"
    exit 1
fi

# Create shim directory and set permissions
mkdir -p "$SHIM_DIR"
chmod 755 "$SHIM_DIR"

# Install master shim binary
MASTER_SHIM="$SHIM_DIR/.shimbin"
echo "📦 Installing master shim: $MASTER_SHIM"
install -m 0755 "$TARGET_BINARY" "$MASTER_SHIM"

# Create individual command shims (real copies, not symlinks)
echo "🔗 Creating command shims..."
for cmd in "${COMMANDS[@]}"; do
    echo "  → $cmd"
    install -m 0755 "$MASTER_SHIM" "$SHIM_DIR/$cmd"
done

echo ""
echo "✅ Shims staged successfully in $SHIM_DIR"
echo "📝 Commands shimmed: ${COMMANDS[*]}"
echo ""

# Display current PATH for reference
echo "📋 Current PATH:"
echo "   $PATH"
echo ""

# Instructions for activation (based on your proven manual approach)
echo "🎯 To activate substrate tracing:"
echo ""
echo "1. Set up clean ORIGINAL_PATH:"
CLEAN_PATH="$HOME/.nvm/versions/node/v22.16.0/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.bun/bin"
echo "   export ORIGINAL_PATH=\"$CLEAN_PATH\""
echo ""
echo "2. Update PATH to include shim directory:"
echo "   export PATH=\"$SHIM_DIR:\$ORIGINAL_PATH\""
echo "   hash -r"
echo ""
echo "3. For non-interactive shells (Claude Code integration):"
echo "   scripts/create_bashenv.sh"
echo "   export BASH_ENV=\"\$HOME/.substrate_bashenv\""
echo ""
echo "4. Set up logging:"
echo "   export TRACE_LOG_FILE=\"\$HOME/.trace_shell.jsonl\""
echo ""
echo "🧪 Test the installation:"
echo "   PATH=\"$SHIM_DIR:\$ORIGINAL_PATH\" bash -lc 'hash -r; which -a git; type -a git; git --version'"
echo ""
echo "📈 Performance note: Cache reduces stat() calls by ~40% after warmup"
echo "🔒 Security: Log files created with 0o600 permissions (user-only access)"