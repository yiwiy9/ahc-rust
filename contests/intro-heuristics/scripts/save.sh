#!/bin/sh
set -eu
case "${1:-}" in
  -h|--help) echo 'Usage: ./scripts/save.sh <solver> <name> [cases=10] [extra options...]'; exit 0 ;;
esac
if [ "$#" -lt 2 ]; then
  echo 'Usage: ./scripts/save.sh <solver> <name> [cases=10] [extra options...]' >&2
  exit 2
fi
script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
solver="$1"
name="$2"
shift 2
cases="${1:-10}"
if [ "$#" -gt 0 ]; then shift; fi
# 採点とソース保存は同期実行。成功表示を見てから次の案へ進む。
exec "$script_dir/../ahc" save "$name" --solver "$solver" --cases "$cases" --builtin "$@"
