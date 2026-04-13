# Prompt Caching Fix

## Problem

The app was **NOT** correctly preserving prompt caching when used as a passthrough for the official Anthropic API. Specifically:

- ❌ **Missing `cache_control` field** in `ContentBlock::Text` struct
- ❌ Prompt caching in message content blocks was silently stripped out  
- ❌ Only system prompt caching was preserved (in `SystemBlock`)

## Root Cause

The `ContentBlock::Text` variant only had a `text` field:

```rust
// BEFORE (broken)
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },  // ❌ Missing cache_control field!
    // ... other variants
}
```

This meant any `cache_control` field in message content blocks would be lost during serialization/deserialization.

## Solution

Added the missing `cache_control` field to `ContentBlock::Text`:

```rust
// AFTER (fixed)
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { 
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<serde_json::Value>,  // ✅ Added!
    },
    // ... other variants
}
```

## Changes Made

### 1. Core Model Update
- **File**: `src/models/mod.rs`
- **Change**: Added `cache_control: Option<serde_json::Value>` to `ContentBlock::Text`

### 2. Pattern Matching Updates
Updated all locations where `ContentBlock::Text` is destructured to handle the new field:

- `src/providers/anthropic_compatible.rs` - Fixed pattern matching with `..` wildcard
- `src/providers/gemini.rs` - Updated response construction with `cache_control: None`
- `src/providers/openai.rs` - Updated parsing and construction
- `src/router/mod.rs` - Fixed text extraction pattern matching  
- `src/server/openai_compat.rs` - Updated OpenAI compatibility layer

### 3. Test Coverage
Added comprehensive tests in `tests/prompt_caching_test.rs`:

- `test_prompt_caching_preservation()` - Verifies cache_control preservation in system and message blocks
- `test_passthrough_caching_behavior()` - Verifies serialization behavior

## Verification

✅ **All tests pass**:
- Cache control headers are preserved in message content blocks
- System prompt caching continues to work  
- Serialization respects `skip_serializing_if` (None values omitted)
- Passthrough behavior maintains original Anthropic API format

## Impact

**Before**:
- System prompt caching: ✅ Works  
- Message content caching: ❌ **Silently stripped**
- Passthrough behavior: ❌ **Broken for caching**

**After**:
- System prompt caching: ✅ Works
- Message content caching: ✅ **Now works correctly**
- Passthrough behavior: ✅ **Fully functional**

## Example

This request now works correctly:

```json
{
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "This content will be cached",
          "cache_control": {"type": "ephemeral"}
        }
      ]
    }
  ],
  "system": [
    {
      "type": "text", 
      "text": "System prompt",
      "cache_control": {"type": "ephemeral"}
    }
  ]
}
```

Both the system and message content `cache_control` fields are now preserved through the passthrough.

## Files Modified

- `src/models/mod.rs` - Core model fix
- `src/providers/anthropic_compatible.rs` - Pattern matching updates
- `src/providers/gemini.rs` - Response construction updates
- `src/providers/openai.rs` - Parsing/construction updates  
- `src/router/mod.rs` - Text extraction updates
- `src/server/openai_compat.rs` - Compatibility layer updates
- `tests/prompt_caching_test.rs` - New test coverage

## Notes

- The fix is backward compatible (adds optional field)
- Uses `skip_serializing_if = "Option::is_none"` to keep output clean
- All existing functionality continues to work
- Full test coverage prevents regressions