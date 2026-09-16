#!/bin/sh
set -eu

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
# 互換入口。ABC側の既存libへリンクするだけで、cloneやpullはしない。
exec python3 "$script_dir/setup-references.py" "$@"
