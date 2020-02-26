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
#[derive(Debug, PartialEq)]
struct Config {
    json: bool,
    check: bool,
    input: Option<String>,
    max_todos: Option<usize>,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let config = match parse_args(&args) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("error: {message}\n{}", usage());
            std::process::exit(2);
        }
    };
    let mut source = String::new();
    let read_result = match config.input.as_deref() {
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
    if config.json {
        println!("{{\"bytes\":{},\"lines\":{},\"functions\":{},\"tests\":{},\"unsafe_mentions\":{},\"unwraps\":{},\"todos\":{}}}", r.bytes, r.lines, r.functions, r.tests, r.unsafe_mentions, r.unwraps, r.todos);
    } else {
        report(&r);
    }
    let todo_limit = config.max_todos.unwrap_or(0);
    if config.check && (r.functions == 0 || r.todos > todo_limit) {
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
    "usage: is-this-rust-yet [--json] [--check] [--max-todos N] [FILE|-]"
}
fn parse_args(args: &[String]) -> Result<Config, String> {
    let mut json = false;
    let mut check = false;
    let mut input = None;
    let mut max_todos = None;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--json" => json = true,
            "--check" => check = true,
            "--max-todos" => {
                i += 1;
                if i == args.len() { return Err("--max-todos requires N".into()); }
                let value = &args[i];
                let limit = value.parse::<usize>().map_err(|_| "--max-todos must be an integer from 0 to 1000000")?;
                if limit > 1_000_000 { return Err("--max-todos must be an integer from 0 to 1000000".into()); }
                max_todos = Some(limit);
            }
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
        i += 1;
    }
    if max_todos.is_some() && !check { return Err("--max-todos requires --check".into()); }
    Ok(Config { json, check, input, max_todos })
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
            Config { json: true, check: true, input: Some("file.rs".into()), max_todos: None }
        );
    }
    #[test]
    fn bad_cli_is_rejected() {
        assert!(parse_args(&["--wat".into()]).is_err());
        assert!(parse_args(&["a.rs".into(), "b.rs".into()]).is_err());
    }
    #[test]
    fn max_todos_requires_check_and_is_bounded() {
        let args = vec!["--check".into(), "--max-todos".into(), "2".into()];
        assert_eq!(parse_args(&args).unwrap().max_todos, Some(2));
        assert!(parse_args(&["--max-todos".into(), "2".into()]).is_err());
        assert!(parse_args(&["--check".into(), "--max-todos".into(), "1000001".into()]).is_err());
        assert!(parse_args(&["--check".into(), "--max-todos".into(), "-1".into()]).is_err());
    }
    #[test]
    fn max_todos_allows_threshold_but_still_requires_function() {
        let report = inspect("fn main() {}\n// TODO\n// TODO\n");
        assert!(report.todos <= 2);
        assert_eq!(report.functions, 1);
    }
}
