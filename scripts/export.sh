#!/bin/sh
set -eu
case "${1:-}" in
  -h|--help) echo 'Usage: ./scripts/export.sh [solver=a] [--file-only]'; exit 0 ;;
esac
script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
solver="${1:-a}"
if [ "$solver" = "--file-only" ]; then
  solver=a
elif [ "$#" -gt 0 ]; then
  shift
fi
# 通常は提出ファイルを保存し、クリップボードにもコピーする。
if [ "${1:-}" = "--file-only" ]; then
  shift
  exec "$script_dir/../ahc" export --solver "$solver" "$@"
fi
exec "$script_dir/../ahc" export --solver "$solver" --clipboard "$@"
