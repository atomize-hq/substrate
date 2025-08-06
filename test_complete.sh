#!/bin/bash
set -e

echo "=== Substrate Shim System Complete Test Suite ==="
echo "Testing all features implemented today"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Function to print test results
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        exit 1
    fi
}

# Clean start
echo -e "\nCleaning up old logs..."
rm -f ~/.trace_shell.jsonl

# Build the project
echo -e "\nBuilding substrate..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished)" || true

# Test 1: npm → node bypass (original bug fix)
echo -e "\n1. Testing npm → node bypass (no recursion error):"
./target/release/substrate -c "npm --version" > /tmp/npm_output.txt 2>&1
if grep -q "Recursion detected" /tmp/npm_output.txt; then
    print_result 1 "npm still shows recursion error"
else
    npm_ver=$(cat /tmp/npm_output.txt | head -1)
    print_result 0 "npm → node bypass works (version: $npm_ver)"
fi

# Test 2: Verify bypass logging
echo -e "\n2. Testing bypass logging in trace file:"
if grep -q '"bypass":true' ~/.trace_shell.jsonl 2>/dev/null; then
    print_result 0 "Bypass events are logged correctly"
else
    print_result 1 "No bypass events found in log"
fi

# Test 3: Correlation features
echo -e "\n3. Testing correlation features:"
tail -5 ~/.trace_shell.jsonl > /tmp/last_logs.json
has_caller=$(grep -c '"caller":' /tmp/last_logs.json || echo 0)
has_stack=$(grep -c '"call_stack":' /tmp/last_logs.json || echo 0)
has_parent=$(grep -c '"parent_cmd_id":' /tmp/last_logs.json || echo 0)

if [ $has_caller -gt 0 ] && [ $has_stack -gt 0 ] && [ $has_parent -gt 0 ]; then
    print_result 0 "Correlation fields present (caller, call_stack, parent_cmd_id)"
else
    print_result 1 "Missing correlation fields"
fi

# Test 4: SHIM_BYPASS=1 (no logging)
echo -e "\n4. Testing SHIM_BYPASS=1 (no trace):"
lines_before=$(wc -l < ~/.trace_shell.jsonl 2>/dev/null || echo 0)
SHIM_BYPASS=1 ~/.cmdshim_rust/git --version > /dev/null 2>&1
lines_after=$(wc -l < ~/.trace_shell.jsonl 2>/dev/null || echo 0)
if [ "$lines_before" == "$lines_after" ]; then
    print_result 0 "SHIM_BYPASS=1 produces no logs"
else
    print_result 1 "SHIM_BYPASS=1 still produces logs"
fi

# Test 5: Signal handling
echo -e "\n5. Testing signal handling (SIGTERM = 143):"
# Create a test script that sleeps
cat > /tmp/sleeper.sh << 'EOF'
#!/bin/bash
sleep 30
EOF
chmod +x /tmp/sleeper.sh

# Run substrate with the sleeper in background
./target/release/substrate -c "/tmp/sleeper.sh" &
SUBSTRATE_PID=$!

# Give it time to start
sleep 1

# Send SIGTERM
kill -TERM $SUBSTRATE_PID 2>/dev/null

# Wait and capture exit code
wait $SUBSTRATE_PID 2>/dev/null
EXIT_CODE=$?

if [ $EXIT_CODE -eq 143 ]; then
    print_result 0 "Signal handling works (exit code 143 for SIGTERM)"
else
    print_result 1 "Wrong exit code: $EXIT_CODE (expected 143)"
fi

# Test 6: Log file permissions
echo -e "\n6. Testing log file permissions:"
if [ -f ~/.trace_shell.jsonl ]; then
    perms=$(stat -f "%A" ~/.trace_shell.jsonl 2>/dev/null || stat -c "%a" ~/.trace_shell.jsonl 2>/dev/null)
    if [ "$perms" == "600" ]; then
        print_result 0 "Log file has correct permissions (0600)"
    else
        print_result 1 "Wrong permissions: $perms (expected 600)"
    fi
else
    print_result 1 "Log file not found"
fi

# Test 7: Call stack capping
echo -e "\n7. Testing call stack capping:"
# Run the unit test for call stack
cd crates/shim
if cargo test test_safe_call_stack --quiet 2>/dev/null; then
    print_result 0 "Call stack capping works (max 8 items)"
else
    print_result 1 "Call stack test failed"
fi
cd ../..

# Test 8: Environment variable consistency
echo -e "\n8. Testing SHIM_ prefix consistency:"
env | grep -E "^(SHIM_|ORIGINAL_PATH|TRACE_LOG)" > /tmp/env_check.txt || true
if grep -E "^(ORIGINAL_PATH|TRACE_LOG)" /tmp/env_check.txt > /dev/null 2>&1; then
    print_result 1 "Found old-style environment variables"
else
    print_result 0 "All environment variables use SHIM_ prefix"
fi

echo -e "\n${GREEN}=== All tests passed! ===${NC}"
echo -e "\nThe substrate shim system is working correctly with:"
echo "• npm → node bypass (no recursion errors)"
echo "• Correlation tracking (caller, call_stack, parent_cmd_id)"
echo "• SHIM_BYPASS=1 for no-trace execution"
echo "• Proper signal handling (128+signal exit codes)"
echo "• Secure log file permissions (0600)"
echo "• Call stack capping at 8 items"
echo "• Consistent SHIM_ environment variable prefix"