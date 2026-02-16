# logcheck-filter CLI Reference

This document contains the help content for the `logcheck-filter` command-line program.

**Command Overview:**

* [`logcheck-filter`↴](#logcheck-filter)
* [`logcheck-filter file`↴](#logcheck-filter-file)
* [`logcheck-filter stdin`↴](#logcheck-filter-stdin)
* [`logcheck-filter journald`↴](#logcheck-filter-journald)

## `logcheck-filter`

Filter logs using logcheck rules

**Usage:** `logcheck-filter [OPTIONS] --rules <RULES> <COMMAND>`

###### **Subcommands:**

* `file` — Read from a file
* `stdin` — Read from standard input
* `journald` — Read from systemd journal

###### **Options:**

* `--rules <RULES>` — Path to logcheck rules directory
* `--format <FORMAT>` — Output format

  Default value: `text`

  Possible values:
  - `text`:
    Human-readable text format
  - `json`:
    JSON format

* `--show <SHOW>` — What entries to show

  Default value: `all`

  Possible values:
  - `all`:
    Show all log entries
  - `violations`:
    Show only violations (cracking/violations)
  - `unmatched`:
    Show only unmatched entries

* `--stats` — Show processing statistics
* `--color` — Enable colored output
* `--output-file <OUTPUT_FILE>` — Write filtered logs to file



## `logcheck-filter file`

Read from a file

**Usage:** `logcheck-filter file <PATH>`

###### **Arguments:**

* `<PATH>` — Path to log file



## `logcheck-filter stdin`

Read from standard input

**Usage:** `logcheck-filter stdin`



## `logcheck-filter journald`

Read from systemd journal

**Usage:** `logcheck-filter journald [OPTIONS]`

###### **Options:**

* `--unit <UNIT>` — Filter by systemd unit
* `--follow` — Follow new journal entries
* `--lines <LINES>` — Show last N entries



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
