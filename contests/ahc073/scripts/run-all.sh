#!/bin/sh
set -eu
case "${1:-}" in
  -h|--help) echo 'Usage: ./scripts/run-all.sh [solver=a] [cases=10] [--parallel] [extra options...]'; exit 0 ;;
esac
script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
solver="${1:-a}"
if [ "$#" -gt 0 ]; then shift; fi
cases="${1:-10}"
if [ "$#" -gt 0 ]; then shift; fi
# 通常は追加ツール不要の逐次計測。並列化も同じスクリプトから選べる。
if [ "${1:-}" = "--parallel" ]; then
  shift
  exec "$script_dir/../ahc" bench --solver "$solver" --cases "$cases" "$@"
fi
exec "$script_dir/../ahc" bench --solver "$solver" --cases "$cases" --builtin "$@"
