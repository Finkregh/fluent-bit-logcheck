# fluent-bit-logcheck-filter - Arch Linux Package

This package provides a Fluent-Bit WASM filter for filtering logs using logcheck rules.

## Building

```bash
makepkg -si
```

## Contents

- **WASM filter**: `/usr/lib/fluent-bit/filters/logcheck_filter.wasm`
- **Documentation**: `/usr/share/doc/fluent-bit-logcheck-filter/`
- **Examples**: `/usr/share/doc/fluent-bit-logcheck-filter/examples/`

## Usage

Add to your Fluent-Bit configuration:

```ini
[FILTER]
    Name  wasm
    Match *
    WASM_Path /usr/lib/fluent-bit/filters/logcheck_filter.wasm
    Function filter_log
    accessible_paths /etc/logcheck
```

See the example configuration:

```bash
cat /usr/share/doc/fluent-bit-logcheck-filter/examples/example-fluent-bit.conf
```

## Integration with logcheck rules

The filter uses standard logcheck rule directories. Install the `logcheck` package or copy rules to `/etc/logcheck/`:

```
/etc/logcheck/
├── cracking.d/          # Active intrusion attempts
├── violations.d/        # Security policy violations
├── violations.ignore.d/ # Known violation exceptions
└── ignore.d.server/     # Normal server operations
```

See `/usr/share/doc/fluent-bit-logcheck-filter/LOGCHECK-INTEGRATION.md` for details.

## Architecture

This package builds WASM bytecode which is architecture-independent (`arch=any`).

## License

Apache-2.0 OR MIT
