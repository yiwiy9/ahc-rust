#!/bin/sh
set -eu

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "Usage: ./scripts/check-linux.sh <contest-id> [solver]" >&2
  exit 2
fi

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
root_dir="$(dirname -- "$script_dir")"
contest_id="$1"
solver="${2:-a}"
contest_dir="$root_dir/contests/$contest_id"

if [ ! -f "$contest_dir/Cargo.toml" ]; then
  echo "Contest not found: $contest_dir" >&2
  exit 1
fi

docker run --rm \
  --user "$(id -u):$(id -g)" \
  --volume "$contest_dir:/workspace" \
  --workdir /workspace \
  --env CARGO_HOME=/tmp/cargo \
  --env CARGO_TARGET_DIR=/tmp/target \
  rust:1.89-bookworm \
  cargo check --locked --bin "$solver"
