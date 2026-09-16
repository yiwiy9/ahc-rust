#!/bin/sh
set -eu
case "${1:-}" in
  -h|--help) echo "Usage: $0 [solver=a] [cases=10] [extra ahc options...]"; exit 0 ;;
esac
script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
solver="${1:-a}"
if [ "$#" -gt 0 ]; then shift; fi
cases="${1:-10}"
if [ "$#" -gt 0 ]; then shift; fi
# 追加ツール不要の逐次実行。並列比較は ./ahc bench を使う。
exec "$script_dir/../ahc" bench --solver "$solver" --cases "$cases" --builtin "$@"
