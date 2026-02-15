# logcheck-filter - Arch Linux Package

This package provides the `logcheck-filter` CLI tool for filtering logs using logcheck rules.

## Building

```bash
makepkg -si
```

## Contents

- **Binary**: `/usr/bin/logcheck-filter`
- **Man page**: `/usr/share/man/man1/logcheck-filter.1`
- **Shell completions**: bash, fish, zsh
- **Documentation**: `/usr/share/doc/logcheck-filter/`
- **Examples**: `/usr/share/doc/logcheck-filter/examples/`

## Usage

```bash
# Filter a log file
logcheck-filter /var/log/syslog

# Read from stdin
journalctl -b | logcheck-filter -

# Use custom rules directory
logcheck-filter --rules /etc/logcheck /var/log/syslog

# JSON output
logcheck-filter --format json /var/log/syslog
```

See `man logcheck-filter` for full documentation.

## Notes

- This package is optimized with `target-cpu=native` for maximum performance on your system
- The binary may not work on systems with older CPUs than the build system
- For a portable build, remove the `RUSTFLAGS` line from PKGBUILD

## License

Apache-2.0 OR MIT
