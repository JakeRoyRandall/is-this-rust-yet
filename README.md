# Is This Rust Yet?

A small humorous source-text inspector. Created in September 2026 as retrospective calendar artwork; this is not historical work from 2020.

Build with `rustc --edition 2021 main.rs -o is-this-rust-yet`. Run `./is-this-rust-yet [--json] [--check] [--max-todos N] [FILE|-]`. Without a file, it reads stdin.

`--check` exits 1 when no function marker is found or TODOs exceed the allowance (zero by default). `--max-todos N` requires `--check` and accepts 0 through 1,000,000. Reports retain actual counts. Argument/read errors exit 2.

This is literal text matching, not Rust parsing, compilation, static analysis, or a security assessment. Counts may include comments and strings. Byte counts are UTF-8 bytes.

Run unit tests with `rustc --edition 2021 --test main.rs -o /tmp/rust-yet-tests && /tmp/rust-yet-tests`.

Input from a file or stdin must be valid UTF-8 and at most 1,048,576 bytes (1 MiB). The reader consumes at most the limit plus one byte; oversized or malformed input exits with status 2.

`--show-todos` prints 1-based lines containing literal uppercase TODO text. With `--json`, it adds a `todo_lines` array of line/text objects. Multiple TODO mentions on one line appear once in this list but remain separate in the total count. This is text matching, not Rust parsing.

`--max-unwraps N` requires `--check` and limits literal unwrap mentions to N (0–1,000,000). Without it, unwraps do not affect the exit status. The function requirement and TODO limit still apply.
