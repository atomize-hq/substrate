---
language:
- en
license: mit
annotations_creators:
- machine-generated
language_creators:
- machine-generated
tags:
- format:agent-traces
- agent-traces
- coding-agent
- codex
- redacted
size_categories:
- 10K<n<100K
task_categories:
- text-generation
task_ids:
- conversational-text-generation
pretty_name: RangaPrasath Coding Sessions
dataset_info:
  features:
  - name: id
    dtype: string
  - name: messages
    dtype:
    - name: role
      dtype: string
    - name: content
      dtype: string
    - name: tool_calls
      dtype:
      - name: name
        dtype: string
      - name: arguments
        dtype: string
  - name: tools
    dtype:
    - name: name
      dtype: string
    - name: description
      dtype: string
    - name: parameters
      dtype: string
  - name: source
    dtype: string
  splits:
  - name: train
    num_bytes: 0
    num_examples: 73
  download_size: 0
  dataset_size: 0
---

# RangaPrasath Coding Sessions

## Dataset Description

This dataset contains **73 real coding sessions** from **OpenAI Codex**, exported using the [pi-brain](https://github.com/0xSero/pi-brain) tool as part of the [Sybil Solutions 20T Session Data Drive](https://training.sybilsolutions.ai/).

### Source Tool
- **Exporter**: pi-brain v0.1.0
- **Source**: OpenAI Codex (`~/.codex/sessions/`)
- **Export date**: 2026-05-16

### What's Included
- **73 sessions** with full trajectories
- **22,528 messages** across all sessions
- Complete tool calls, code edits, test output, and review loops
- Sessions span from February 2026

### Redaction Applied

All sessions have been processed through pi-brain's deterministic privacy engine:
- API keys and provider tokens replaced with `<API_KEY_N>` placeholders
- Emails replaced with `<EMAIL_N>` placeholders
- Phone numbers, JWTs, auth headers, and IPs redacted
- Filesystem paths anonymized
- Timestamps fuzzed to prevent correlation
- All redaction runs **locally** before any data leaves the machine

### Data Format

The dataset is provided in JSONL format with the following schema:
```json
{
  "id": "anonymized-session-id",
  "messages": [
    {
      "role": "user|assistant|system|tool",
      "content": "...",
      "tool_calls": [...]
    }
  ],
  "tools": [...],
  "source": "codex"
}
```

### License
MIT

### Citation
If you use this dataset, please reference:
```
@misc{rangaprasath-coding-sessions-2026,
  title={RangaPrasath Coding Sessions},
  author={RangaPrasath},
  year={2026},
  url={https://huggingface.co/datasets/RangaPrasath/coding-sessions},
  note={Exported via pi-brain for Sybil Solutions 20T Session Data Drive}
}
```
