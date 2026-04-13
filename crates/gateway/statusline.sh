#!/bin/bash
# Substrate Gateway Statusline Script
# Shows models used in recent requests with sparkline bars
#
# Installed via: substrate-gateway install-statusline
# File location: ~/.substrate-gateway/statusline.sh
#
# Displays: model@provider ████ model2@provider ██
# Each █ = 1 request (out of last 20)

# Only show gateway info if Claude Code is using the gateway (ANTHROPIC_BASE_URL set)
if [ -z "$ANTHROPIC_BASE_URL" ]; then
    exit 0
fi

GATEWAY_FILE="$HOME/.substrate-gateway/last_routing.json"

if [ ! -f "$GATEWAY_FILE" ]; then
    echo "Substrate Gateway: no routing yet"
    exit 0
fi

# Read recent requests array
RECENT=$(jq -r '.recent // []' "$GATEWAY_FILE")

if [ "$RECENT" = "[]" ] || [ -z "$RECENT" ]; then
    # Fallback: show current model
    MODEL=$(jq -r '.model // "unknown"' "$GATEWAY_FILE")
    PROVIDER=$(jq -r '.provider // "unknown"' "$GATEWAY_FILE")
    echo "$MODEL@$PROVIDER"
    exit 0
fi

# Strip date suffixes and long model paths (keep last 2 segments), get models in recency order
STRIPPED=$(echo "$RECENT" | jq -r '.[]' | sed -e 's/-[0-9]\{8\}@/@/' -e 's|.*/\([^/]*/[^@]*\)@|\1@|')
UNIQUE_MODELS=$(echo "$STRIPPED" | awk 'seen[$0]==0 {print; seen[$0]=1; count++} count>=3 {exit}')

# Build output: models in recency order, fixed-width bars show proportion
BAR_WIDTH=10
TOTAL=20
OUTPUT=""
while read -r MODEL; do
    [ -z "$MODEL" ] && continue
    COUNT=$(echo "$STRIPPED" | grep -cx "$MODEL")

    # Calculate filled portion (at least 1 if count > 0)
    FILLED=$(( (COUNT * BAR_WIDTH + TOTAL - 1) / TOTAL ))  # round up
    [ "$FILLED" -gt "$BAR_WIDTH" ] && FILLED=$BAR_WIDTH
    [ "$FILLED" -lt 1 ] && FILLED=1
    HOLLOW=$((BAR_WIDTH - FILLED))

    BAR=$(printf '█%.0s' $(seq 1 $FILLED))$(printf '░%.0s' $(seq 1 $HOLLOW) 2>/dev/null)

    if [ -n "$OUTPUT" ]; then
        OUTPUT="$OUTPUT $MODEL $BAR"
    else
        OUTPUT="$MODEL $BAR"
    fi
done <<< "$UNIQUE_MODELS"

echo "$OUTPUT"
