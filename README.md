# Is This Rust Yet? — core edition

Created September 2026 as a retrospective exercise, not historical 2020 work. This is the first runnable increment of **Is This Rust Yet?**, a deliberately tiny source-text joke utility.

Run without Cargo or dependencies:

```sh
rustc --edition=2021 main.rs -o /tmp/is-this-rust-yet-core
printf 'fn main() {}\n' | /tmp/is-this-rust-yet-core
```

It reads one file argument, `-`, or stdin and reports literal counts for bytes, lines, `fn`, tests, `unsafe`, `unwrap`, and `TODO`, followed by a checklist and a playful verdict.

Invalid options, multiple inputs, and unreadable files return exit code 2. Counts are substring matches, including comments and strings; this is not a compiler or static analyzer.

Git author dates are deliberately assigned for calendar artwork; actual creation was September 2026 and committer timestamps record that creation.
