#!/bin/bash

echo "Testing signal handling separately..."

# Start substrate with sleep command
./target/release/substrate -c "sleep 10" &
PID=$!

# Give it a moment to start
sleep 0.5

# Check if process is running
if ps -p $PID > /dev/null 2>&1; then
    echo "Process started with PID $PID"
    
    # Send SIGTERM
    echo "Sending SIGTERM..."
    kill -TERM $PID
    
    # Wait for it to exit
    wait $PID 2>/dev/null
    EXIT_CODE=$?
    
    echo "Exit code: $EXIT_CODE"
    
    if [ $EXIT_CODE -eq 143 ]; then
        echo "✓ Correct exit code for SIGTERM (143 = 128+15)"
    else
        echo "✗ Wrong exit code: $EXIT_CODE (expected 143)"
    fi
else
    echo "Process didn't start properly"
fi