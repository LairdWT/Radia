#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

if find "$repo_root/crates" -type f \( -name '*.rs' -o -name '*.wgsl' \) \
    -exec grep -nE '(^|[^[:alnum:]_])(Mat([2-4]|[A-Z])[A-Za-z0-9_]*|mat[2-4]x[2-4])([^[:alnum:]_]|$)' {} +
then
    printf '%s\n' 'matrix-ban: forbidden matrix type found' >&2
    exit 1
fi

file_count=$(find "$repo_root/crates" -type f \( -name '*.rs' -o -name '*.wgsl' \) | wc -l | tr -d ' ')
printf 'matrix-ban: files=%s findings=0\n' "$file_count"
