#!/bin/bash

echo "Testing LiteLLM /v1/messages endpoint..."
curl -X POST http://localhost:4000/v1/messages \
  -H "x-litellm-api-key: chanhee" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-5",
    "max_tokens": 50,
    "messages": [{"role": "user", "content": "Say hi"}]
  }'
