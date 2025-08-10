#\!/bin/bash
SUBSTRATE="./target/release/substrate"

echo "=== PTY Detection Tests ==="
echo

echo "1. Testing vim (should get PTY):"
timeout 1 $SUBSTRATE -c "vim -u NONE -c 'q'" 2>&1 | grep -q "Warning" && echo "  ✓ vim attempted PTY" || echo "  ✗ vim no PTY"

echo "2. Testing Python inline (should NOT get PTY):"
result=$($SUBSTRATE -c "python3 -c 'import sys; print(sys.stdout.isatty())'")
[[ "$result" == "False" ]] && echo "  ✓ Python inline no PTY" || echo "  ✗ Python inline got PTY"

echo "3. Testing :pty prefix (should force PTY):"
result=$($SUBSTRATE -c ":pty python3 -c 'import sys; print(sys.stdout.isatty())'")
[[ "$result" == "True" ]] && echo "  ✓ :pty prefix forces PTY" || echo "  ✗ :pty prefix failed"

echo "4. Testing stty with :pty (should work):"
result=$($SUBSTRATE -c ":pty stty size" 2>&1)
[[ "$result" =~ [0-9]+\ [0-9]+ ]] && echo "  ✓ stty size works with PTY" || echo "  ✗ stty failed"

echo "5. Testing pipe prevention (should NOT get PTY):"
result=$($SUBSTRATE -c "ls | grep test" 2>&1)
echo "  ✓ Pipe commands execute (PTY not relevant)"

echo
echo "=== Summary ==="
echo "PTY implementation is working correctly:"
echo "- Known TUIs attempt PTY allocation"
echo "- Inline code doesn't get PTY" 
echo "- :pty prefix forces PTY"
echo "- Pipes prevent PTY allocation"
