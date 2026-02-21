#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

declare -a packages=()

if [[ "$#" -gt 0 ]]; then
    packages=("$@")
else
    packages=(
        "logcheck-filter-git"
        "logcheck-filter-bin"
        "fluent-bit-logcheck-filter-git"
        "fluent-bit-logcheck-filter-bin"
        "logcheck-database"
    )
fi

for pkg in "${packages[@]}"; do
    pkg_dir="$root_dir/$pkg"
    if [[ ! -f "$pkg_dir/PKGBUILD" ]]; then
        echo "Missing PKGBUILD for $pkg. Run ./pkgbuild/generate-pkgbuilds.sh first." >&2
        exit 1
    fi

    (cd "$pkg_dir" && makepkg --printsrcinfo >.SRCINFO)
done
