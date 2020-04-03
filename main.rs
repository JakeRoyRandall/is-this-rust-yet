use std::{
    env, fs,
    io::{self, Read},
};

#[derive(Debug, PartialEq)]
struct Report {
    bytes: usize,
    lines: usize,
    functions: usize,
    tests: usize,
    unsafe_mentions: usize,
    unwraps: usize,
    todos: usize,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let (json, check, input) = match parse_args(&args) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("error: {message}\n{}", usage());
            std::process::exit(2);
        }
    };
    let mut source = String::new();
    let read_result = match input.as_deref() {
        None | Some("-") => io::stdin().read_to_string(&mut source),
        Some(path) => fs::read_to_string(path).map(|text| {
            source = text;
            0
        }),
    };
    if let Err(error) = read_result {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
    let r = inspect(&source);
    if json {
        println!("{{\"bytes\":{},\"lines\":{},\"functions\":{},\"tests\":{},\"unsafe_mentions\":{},\"unwraps\":{},\"todos\":{}}}", r.bytes, r.lines, r.functions, r.tests, r.unsafe_mentions, r.unwraps, r.todos);
    } else {
        report(&r);
    }
    if check && (r.functions == 0 || r.todos > 0) {
        std::process::exit(1);
    }
}
fn print_help() {
    println!(
        "{}\nCounts text patterns only; it is not rustc or static analysis.",
        usage()
    );
}
fn usage() -> &'static str {
    "usage: is-this-rust-yet [--json] [--check] [FILE|-]"
}
fn parse_args(args: &[String]) -> Result<(bool, bool, Option<String>), String> {
    let mut json = false;
    let mut check = false;
    let mut input = None;
    for arg in args {
        match arg.as_str() {
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--json" => json = true,
            "--check" => check = true,
            "-" => {
                if input.is_some() {
                    return Err("more than one input: -".into());
                }
                input = Some("-".into());
            }
            value if value.starts_with('-') => return Err(format!("unknown option {value}")),
            value if input.is_some() => return Err(format!("more than one input: {value}")),
            value => input = Some(value.to_string()),
        }
    }
    Ok((json, check, input))
}
fn mark(ok: bool) -> &'static str {
    if ok {
        "[x]"
    } else {
        "[ ]"
    }
}
fn inspect(source: &str) -> Report {
    Report {
        bytes: source.len(),
        lines: source.lines().count(),
        functions: source.matches("fn ").count(),
        tests: source.matches("#[test]").count(),
        unsafe_mentions: source.matches("unsafe").count(),
        unwraps: source.matches("unwrap").count(),
        todos: source.matches("TODO").count(),
    }
}
fn report(r: &Report) {
    println!("Is this Rust yet? tiny source-text inspection\n\nbytes: {}\nlines: {}\nfunctions: {}\ntests: {}\nunsafe mentions: {}\nunwrap mentions: {}\nTODOs: {}\n\nchecklist:\n  {} fn keyword(s): the compiler may recognize a function\n  {} test attribute(s): future-you has backup\n  {} TODO(s): the 2020 lockdown backlog lives\n  {} no unsafe mention: borrow checker vibes are pending\n\nverdict: {}", r.bytes, r.lines, r.functions, r.tests, r.unsafe_mentions, r.unwraps, r.todos, mark(r.functions > 0), mark(r.tests > 0), mark(r.todos > 0), mark(r.unsafe_mentions == 0), if r.functions > 0 { "Rust-shaped. Please ask rustc before ordering a commemorative hoodie." } else { "Rust-adjacent. It may be a README wearing a trench coat." });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_input_is_safe() {
        assert_eq!(
            inspect(""),
            Report {
                bytes: 0,
                lines: 0,
                functions: 0,
                tests: 0,
                unsafe_mentions: 0,
                unwraps: 0,
                todos: 0
            }
        );
    }
    #[test]
    fn unicode_counts_bytes() {
        assert_eq!(inspect("fn main() { println!(\"🦀\"); }").bytes, 31);
    }
    #[test]
    fn flags_can_be_combined_and_reordered() {
        let args = vec!["file.rs".into(), "--check".into(), "--json".into()];
        assert_eq!(
            parse_args(&args).unwrap(),
            (true, true, Some("file.rs".into()))
        );
    }
    #[test]
    fn bad_cli_is_rejected() {
        assert!(parse_args(&["--wat".into()]).is_err());
        assert!(parse_args(&["a.rs".into(), "b.rs".into()]).is_err());
    }
}
