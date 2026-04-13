#!/bin/bash

echo "=== Testing Router Routing Logic ==="
echo

# Test 1: Default routing
echo "1️⃣  Test Default Routing (long message)"
curl -X POST http://localhost:13456/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-5",
    "max_tokens": 50,
    "messages": [
      {"role": "user", "content": "This is a longer message"},
      {"role": "assistant", "content": "Sure, I can help"},
      {"role": "user", "content": "Please explain quantum computing in detail"}
    ]
  }' 2>&1 | grep -E '"model"|error'
echo
echo

# Test 2: Background routing (Haiku model)
echo "2️⃣  Test Background Routing (Haiku model)"
curl -X POST http://localhost:13456/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-3-5-haiku-20241022",
    "max_tokens": 50,
    "messages": [{"role": "user", "content": "Hi"}]
  }' 2>&1 | grep -E '"model"|error'
echo
echo

# Test 3: Think routing
echo "3️⃣  Test Think Routing (thinking enabled)"
curl -X POST http://localhost:13456/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-5",
    "max_tokens": 50,
    "thinking": {
      "type": "enabled",
      "budget_tokens": 10000
    },
    "messages": [{"role": "user", "content": "Solve this complex problem"}]
  }' 2>&1 | grep -E '"model"|error'
echo
echo

# Test 4: WebSearch routing (highest priority)
echo "4️⃣  Test WebSearch Routing (web_search tool)"
curl -X POST http://localhost:13456/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-5",
    "max_tokens": 50,
    "tools": [
      {
        "type": "web_search_2025_04",
        "name": "web_search",
        "description": "Search the web"
      }
    ],
    "messages": [{"role": "user", "content": "Search for latest news"}]
  }' 2>&1 | grep -E '"model"|error'
echo
echo

# Test 5: WebSearch has highest priority (even with thinking)
echo "5️⃣  Test WebSearch Priority (websearch + thinking)"
curl -X POST http://localhost:13456/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-5",
    "max_tokens": 50,
    "thinking": {
      "type": "enabled",
      "budget_tokens": 10000
    },
    "tools": [
      {
        "type": "web_search",
        "name": "search"
      }
    ],
    "messages": [{"role": "user", "content": "Search and think"}]
  }' 2>&1 | grep -E '"model"|error'
echo
echo

echo "=== Test Complete ==="
echo
echo "Expected results:"
echo "1. Default routing → default-model"
echo "2. Background routing (Haiku) → background-model"
echo "3. Think routing → think-model"
echo "4. WebSearch routing → websearch-model"
echo "5. WebSearch priority → websearch-model (not think-model)"
