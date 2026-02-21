# Arch Linux PKGBUILDs

This directory contains **generated** PKGBUILDs for Arch Linux packaging.

## Workflow

PKGBUILDs live in `pkgbuild/templates/*/PKGBUILD`. The generated outputs are
written to `pkgbuild/<package>/PKGBUILD` for build automation and CI.

Generate (or refresh) all PKGBUILDs:

```bash
./pkgbuild/generate-pkgbuilds.sh
```

Update checksums for release-based packages:

```bash
./pkgbuild/update-checksums.sh
```

Update .SRCINFO metadata for all packages:

```bash
./pkgbuild/update-srcinfo.sh
```

## Packages

- `logcheck-filter-git` – VCS build from Git main branch
- `logcheck-filter-bin` – prebuilt CLI from GitHub releases
- `fluent-bit-logcheck-filter-git` – VCS build for WASM filter
- `fluent-bit-logcheck-filter-bin` – prebuilt WASM filter from GitHub releases
- `logcheck-database` – Debian rules repackaged for Arch

## Notes

- The `-bin` packages download release assets and a source tarball to install
  documentation and license files.
- Replace `REPLACE_ME` checksums before building or run `update-checksums.sh`.
