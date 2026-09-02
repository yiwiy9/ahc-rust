#!/bin/sh
set -eu

if [ "$#" -lt 2 ]; then
  echo "Usage: run-isolated.sh <work-directory> <program> [args...]" >&2
  exit 2
fi

work_directory="$1"
shift

mkdir -p "$work_directory"
cd "$work_directory"
exec "$@"
