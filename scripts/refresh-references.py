#!/usr/bin/env python3
"""Compatibility entry point: links need no periodic refresh."""
import runpy
from pathlib import Path

print("参照はシンボリックリンクです。定期的な更新は不要です。")
runpy.run_path(str(Path(__file__).with_name("setup-references.py")), run_name="__main__")
