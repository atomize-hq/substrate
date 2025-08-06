#!/bin/bash
echo "=== Testing Signal Handling (Single Shell) ==="

# Test 1: Basic SIGTERM
echo -e "\n1. Basic SIGTERM test:"
bash -c '
  cd /Users/spensermcconnell/__Active_Code/substrate
  ./target/release/substrate -c "sleep 10" &
  PID=$!
  sleep 1
  kill -TERM $PID 2>/dev/null
  wait $PID 2>/dev/null
  EXIT_CODE=$?
  if [ $EXIT_CODE -eq 143 ]; then
    echo "✓ Exit code 143 (128+15 for SIGTERM)"
  else
    echo "✗ Wrong exit code: $EXIT_CODE"
  fi
'

# Test 2: Process group kill (like Ctrl-C)
echo -e "\n2. Process group kill test:"
bash -c '
  cd /Users/spensermcconnell/__Active_Code/substrate
  ./target/release/substrate -c "sleep 10" &
  PID=$!
  sleep 1
  PGID=$(ps -o pgid= -p $PID 2>/dev/null | tr -d " ")
  if [ ! -z "$PGID" ]; then
    kill -TERM -$PGID 2>/dev/null
    wait $PID 2>/dev/null
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 143 ]; then
      echo "✓ Process group kill works (exit 143)"
    else
      echo "✗ Wrong exit code: $EXIT_CODE"
    fi
  else
    echo "✗ Could not get process group"
  fi
'

# Test 3: SIGINT (Ctrl-C)
echo -e "\n3. SIGINT test:"
bash -c '
  cd /Users/spensermcconnell/__Active_Code/substrate
  ./target/release/substrate -c "sleep 10" &
  PID=$!
  sleep 1
  kill -INT $PID 2>/dev/null
  wait $PID 2>/dev/null
  EXIT_CODE=$?
  if [ $EXIT_CODE -eq 130 ]; then
    echo "✓ Exit code 130 (128+2 for SIGINT)"
  else
    echo "✗ Wrong exit code: $EXIT_CODE"
  fi
'

# Test 4: Check logs for term_signal
echo -e "\n4. Checking logs for term_signal:"
if grep -q '"term_signal":' ~/.trace_shell.jsonl 2>/dev/null; then
    echo "✓ term_signal field found in logs"
    grep '"term_signal":' ~/.trace_shell.jsonl | tail -1 | jq -c '{event_type, exit_code, term_signal}'
else
    echo "Note: No term_signal fields in logs (signals may not have been sent)"
fi

echo -e "\n=== Signal handling tests complete ==="