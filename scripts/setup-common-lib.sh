#!/bin/sh
set -eu

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
workspace_dir="$(dirname -- "$(dirname -- "$script_dir")")"
destination="$workspace_dir/atcoder-lib"
repository="https://github.com/yiwiy9/practice-algorithm-rust-snippets.git"

if [ -e "$destination" ]; then
  echo "Kept existing: $destination"
  exit 0
fi

git clone "$repository" "$destination"
echo "Created: $destination"
