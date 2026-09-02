#!/bin/sh
set -eu

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
root_dir="$(dirname -- "$script_dir")"
destination="$root_dir/.tools"
version="0.4.0"

if [ -x "$destination/bin/pahcer" ] && \
  "$destination/bin/pahcer" --version | grep -q "pahcer $version"; then
  echo "pahcer $version is already installed in $destination"
  exit 0
fi

echo "Installing pahcer $version inside this workspace..."
cargo install pahcer \
  --version "$version" \
  --locked \
  --root "$destination" \
  --force

echo "Installed: $destination/bin/pahcer"
