#!/usr/bin/env python3
from pathlib import Path

target_dir = Path("world-mac-smoke")
target_dir.mkdir(parents=True, exist_ok=True)
(target_dir / "file.txt").write_text("data\n", encoding="utf-8")
