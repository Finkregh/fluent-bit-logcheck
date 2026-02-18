#!/usr/bin/env bash
set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
    echo "jq is required to parse release-plz output" >&2
    exit 1
fi

if ! command -v makepkg >/dev/null 2>&1; then
    echo "makepkg is required to generate .SRCINFO" >&2
    exit 1
fi

if [ $# -lt 1 ]; then
    echo "Usage: $0 <release-plz-pr-json>" >&2
    exit 1
fi

pr_json="$1"

if [ ! -f "$pr_json" ]; then
    echo "release-plz PR json not found: $pr_json" >&2
    exit 1
fi

update_pkgbuild() {
    local pkgbuild_dir="$1"
    local version="$2"

    if [ ! -f "$pkgbuild_dir/PKGBUILD" ]; then
        return 0
    fi

    echo "Updating $pkgbuild_dir to version $version"
    sed -i "s/^pkgver=.*/pkgver=${version}/" "$pkgbuild_dir/PKGBUILD"

    (cd "$pkgbuild_dir" && makepkg --printsrcinfo >.SRCINFO)
}

mapfile -t releases < <(jq -c '.releases[]?' "$pr_json")

if [ ${#releases[@]} -eq 0 ]; then
    echo "No releases found in release-plz output." >&2
    exit 1
fi

for release in "${releases[@]}"; do
    package_name=$(jq -r '.package_name' <<<"$release")
    version=$(jq -r '.version' <<<"$release")

    case "$package_name" in
    logcheck-fluent-bit-filter)
        update_pkgbuild "pkgbuild/logcheck-filter" "$version"
        update_pkgbuild "pkgbuild/fluent-bit-logcheck-filter" "$version"
        ;;
    *)
        echo "No PKGBUILD mapping for package: $package_name"
        ;;
    esac
done
