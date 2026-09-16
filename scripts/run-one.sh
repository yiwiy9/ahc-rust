#!/bin/sh
set -eu
case "${1:-}" in
  -h|--help) echo "Usage: $0 [solver=a] [seed=0] [extra ahc options...]"; exit 0 ;;
esac
script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
solver="${1:-a}"
if [ "$#" -gt 0 ]; then shift; fi
seed="${1:-0}"
if [ "$#" -gt 0 ]; then shift; fi
exec "$script_dir/../ahc" run "$seed" --solver "$solver" "$@"
