#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
templates_dir="$root_dir/templates"

declare -a packages=(
    "logcheck-filter-git"
    "logcheck-filter-bin"
    "fluent-bit-logcheck-filter-git"
    "fluent-bit-logcheck-filter-bin"
    "logcheck-database"
)

for pkg in "${packages[@]}"; do
    src="$templates_dir/$pkg/PKGBUILD"
    dst="$root_dir/$pkg/PKGBUILD"

    mkdir -p "$(dirname "$dst")"
    cp "$src" "$dst"
done
