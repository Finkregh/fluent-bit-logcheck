# logcheck-database

This package provides the official Debian logcheck rule database, repackaged for Arch Linux.

## What it provides

The package installs 195+ logcheck rule files to `/etc/logcheck/` organized into categories:

- **`cracking.d/`** - Active intrusion detection rules (security alerts)
- **`violations.d/`** - Security policy violation detection
- **`ignore.d.server/`** - Normal server operations (130+ service rules)
- **`ignore.d.workstation/`** - Normal workstation operations
- **`ignore.d.paranoid/`** - Paranoid mode filters (very strict)
- **`violations.ignore.d/`** - Violations that can be safely ignored

## Usage

Install this package as an optional dependency for:

- `logcheck-filter` - CLI tool for filtering logs using logcheck rules
- `fluent-bit-logcheck-filter` - Fluent-Bit WASM filter using logcheck rules

The rules are automatically available at `/etc/logcheck/` for any logcheck-compatible tools.

## Source

Rules are extracted from the official Debian `logcheck-database` package:
<https://packages.debian.org/testing/logcheck-database>

These rules have been tested and refined across thousands of Debian deployments, have fun using them in archlinux.

## Rule Format

Rules are extended regular expressions (egrep format) that match log line patterns:

```bash
# Example from ignore.d.server/ssh
^\w{3} [ :0-9]{11} \S+ sshd\[\d+\]: Failed password for invalid user \S+ from \S+ port \d+ ssh2$

# Example from ignore.d.server/systemd  
^\w{3} [ :0-9]{11} \S+ systemd\[\d+\]: Started .+$
```

## License

GPL (same as original Debian package)
