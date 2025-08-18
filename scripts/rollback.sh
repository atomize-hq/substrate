#!/bin/bash
set -euo pipefail

# Emergency rollback of substrate shims
# This script safely removes all shim components and restores original environment

SHIM_DIR="$HOME/.substrate/shims"
BASHENV_FILE="$HOME/.substrate_bashenv"
BACKUP_SUFFIX=".DISABLED.$(date +%s)"

echo "🚨 Emergency rollback of substrate shims"
echo ""

# Disable shim directory
if [[ -d "$SHIM_DIR" ]]; then
    echo "📁 Disabling shim directory: $SHIM_DIR -> $SHIM_DIR$BACKUP_SUFFIX"
    mv "$SHIM_DIR" "$SHIM_DIR$BACKUP_SUFFIX"
    echo "✅ Shim directory disabled"
else
    echo "ℹ️  Shim directory not found: $SHIM_DIR"
fi

# Disable BASH_ENV file
if [[ -f "$BASHENV_FILE" ]]; then
    echo "📄 Disabling BASH_ENV file: $BASHENV_FILE -> $BASHENV_FILE$BACKUP_SUFFIX"
    mv "$BASHENV_FILE" "$BASHENV_FILE$BACKUP_SUFFIX"
    echo "✅ BASH_ENV file disabled"
else
    echo "ℹ️  BASH_ENV file not found: $BASHENV_FILE"
fi

# Clear bash command hash table
echo "🧹 Clearing bash command cache..."
hash -r
echo "✅ Command cache cleared"

echo ""
echo "🎯 Manual cleanup steps:"
echo "1. Update your shell configuration files:"
echo "   - ~/.bashrc"
echo "   - ~/.bash_profile" 
echo "   - ~/.zshrc"
echo "   - ~/.zprofile"
echo ""
echo "2. Remove any lines containing:"
echo "   - SHIM_ORIGINAL_PATH"
echo "   - .substrate/shims"
echo "   - .substrate_bashenv"
echo "   - BASH_ENV"
echo ""
echo "3. Start a new shell session or run:"
echo "   source ~/.bashrc  # or your shell's config file"
echo ""
echo "4. Verify commands work normally:"
echo "   which git"
echo "   type git"
echo ""

# Show what was backed up
echo "💾 Backup files created:"
if [[ -d "$SHIM_DIR$BACKUP_SUFFIX" ]]; then
    echo "   $SHIM_DIR$BACKUP_SUFFIX"
fi
if [[ -f "$BASHENV_FILE$BACKUP_SUFFIX" ]]; then
    echo "   $BASHENV_FILE$BACKUP_SUFFIX"
fi
echo ""
echo "🗑️  To permanently remove backups:"
echo "   rm -rf $SHIM_DIR$BACKUP_SUFFIX"
echo "   rm -f $BASHENV_FILE$BACKUP_SUFFIX"
echo ""
echo "✅ Rollback complete - substrate shims are now disabled"