#!/bin/bash
echo "Testing LiteLLM /v1/chat/completions directly..."
curl -X POST http://localhost:4000/v1/chat/completions \
  -H "x-litellm-api-key: chanhee" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-5",
    "max_tokens": 50,
    "messages": [{"role": "user", "content": "Say hi"}]
  }'
