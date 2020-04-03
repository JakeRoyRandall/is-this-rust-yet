# Is This Rust Yet?

Created September 2026 as a retrospective exercise, not historical 2020 work. This increment keeps the core report and adds `--json`, a `--check` CI gate, and unit tests.

Run without Cargo or dependencies:

```sh
rustc --edition=2021 main.rs -o /tmp/is-this-rust-yet-feature
printf 'fn main() {}\n' | /tmp/is-this-rust-yet-feature --json -
rustc --edition=2021 --test main.rs -o /tmp/is-this-rust-yet-feature-tests
/tmp/is-this-rust-yet-feature-tests
sh cli_test.sh
```

`--check` exits 1 if the text has no `fn` or has any `TODO`. These are literal text matches, not parsing or compilation.

Flags may be combined and used in either order with one input path or `-`. Exit codes: 0 for successful reporting/check, 1 for a failed text check, 2 for usage or read errors. Counts include comments and strings; UTF-8 bytes and Rust `str::lines` line semantics are used. No compiler, safety, or performance claims.

Git author dates are deliberately assigned for calendar artwork; actual creation was September 2026 and committer timestamps record that creation.
