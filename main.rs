use std::{
    env, fs,
    io::{self, Read},
};

const MAX_INPUT_BYTES: usize = 1_048_576;

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
    max_unwraps: Option<usize>,
    show_todos: bool,
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
    let source = match config.input.as_deref() {
        None | Some("-") => read_limited(io::stdin()),
        Some(path) => fs::File::open(path).map_err(|error| error.to_string()).and_then(read_limited),
    };
    let source = match source {
        Ok(source) => source,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    };
    let r = inspect(&source);
    if config.json {
        print!("{{\"bytes\":{},\"lines\":{},\"functions\":{},\"tests\":{},\"unsafe_mentions\":{},\"unwraps\":{},\"todos\":{}", r.bytes, r.lines, r.functions, r.tests, r.unsafe_mentions, r.unwraps, r.todos);
        if config.show_todos {
            print!(",\"todo_lines\":[");
            for (index, (line, text)) in todo_lines(&source).iter().enumerate() {
                if index > 0 { print!(","); }
                print!("{{\"line\":{},\"text\":\"{}\"}}", line, json_escape(text));
            }
            print!("]");
        }
        println!("}}");
    } else {
        report(&r);
        if config.show_todos {
            println!("\nTODO lines:");
            let lines = todo_lines(&source);
            if lines.is_empty() { println!("  (none)"); }
            for (line, text) in lines { println!("  {line}: {text}"); }
        }
    }
    let todo_limit = config.max_todos.unwrap_or(0);
    let unwrap_limit = config.max_unwraps.unwrap_or(usize::MAX);
    if config.check && (r.functions == 0 || r.todos > todo_limit || r.unwraps > unwrap_limit) {
        std::process::exit(1);
    }
}
fn read_limited<R: Read>(reader: R) -> Result<String, String> {
    let mut bytes = Vec::with_capacity(MAX_INPUT_BYTES.min(8192));
    reader
        .take((MAX_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(format!("input exceeds {} byte limit", MAX_INPUT_BYTES));
    }
    String::from_utf8(bytes).map_err(|_| "input is not valid UTF-8".into())
}
fn print_help() {
    println!(
        "{}\nCounts text patterns only; it is not rustc or static analysis.",
        usage()
    );
}
fn usage() -> &'static str {
    "usage: is-this-rust-yet [--json] [--check] [--max-todos N] [--max-unwraps N] [--show-todos] [FILE|-]"
}
fn parse_args(args: &[String]) -> Result<Config, String> {
    let mut json = false;
    let mut check = false;
    let mut input = None;
    let mut max_todos = None;
    let mut max_unwraps = None;
    let mut show_todos = false;
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
            "--show-todos" => show_todos = true,
            "--max-todos" => {
                i += 1;
                if i == args.len() { return Err("--max-todos requires N".into()); }
                let value = &args[i];
                let limit = value.parse::<usize>().map_err(|_| "--max-todos must be an integer from 0 to 1000000")?;
                if limit > 1_000_000 { return Err("--max-todos must be an integer from 0 to 1000000".into()); }
                max_todos = Some(limit);
            }
            "--max-unwraps" => {
                i += 1;
                if i == args.len() { return Err("--max-unwraps requires N".into()); }
                let value = &args[i];
                let limit = value.parse::<usize>().map_err(|_| "--max-unwraps must be an integer from 0 to 1000000")?;
                if limit > 1_000_000 { return Err("--max-unwraps must be an integer from 0 to 1000000".into()); }
                max_unwraps = Some(limit);
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
    if max_unwraps.is_some() && !check { return Err("--max-unwraps requires --check".into()); }
    Ok(Config { json, check, input, max_todos, max_unwraps, show_todos })
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
fn todo_lines(source: &str) -> Vec<(usize, String)> {
    source.lines().enumerate().filter_map(|(index, line)| {
        let text = line.strip_suffix('\r').unwrap_or(line);
        text.contains("TODO").then(|| (index + 1, text.to_string()))
    }).collect()
}
fn json_escape(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
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
            Config { json: true, check: true, input: Some("file.rs".into()), max_todos: None, max_unwraps: None, show_todos: false }
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
    fn max_unwraps_requires_check_and_is_bounded() {
        let args = vec!["--check".into(), "--max-unwraps".into(), "2".into()];
        assert_eq!(parse_args(&args).unwrap().max_unwraps, Some(2));
        assert!(parse_args(&["--max-unwraps".into(), "2".into()]).is_err());
        assert!(parse_args(&["--check".into(), "--max-unwraps".into(), "1000001".into()]).is_err());
        assert!(parse_args(&["--check".into(), "--max-unwraps".into(), "-1".into()]).is_err());
    }
    #[test]
    fn max_todos_allows_threshold_but_still_requires_function() {
        let report = inspect("fn main() {}\n// TODO\n// TODO\n");
        assert!(report.todos <= 2);
        assert_eq!(report.functions, 1);
    }
    #[test]
    fn bounded_reader_accepts_exact_limit_and_rejects_over_limit() {
        use std::io::Cursor;
        assert_eq!(read_limited(Cursor::new(vec![b'x'; MAX_INPUT_BYTES])).unwrap().len(), MAX_INPUT_BYTES);
        let error = read_limited(Cursor::new(vec![b'x'; MAX_INPUT_BYTES + 1])).unwrap_err();
        assert!(error.contains("exceeds"));
    }
    #[test]
    fn bounded_reader_rejects_invalid_utf8() {
        use std::io::Cursor;
        assert_eq!(read_limited(Cursor::new(vec![0xff])).unwrap_err(), "input is not valid UTF-8");
    }
    #[test]
    fn todo_lines_are_numbered_and_strip_crlf_separator() {
        assert_eq!(todo_lines("one\r\n// TODO: \"ship\"\r\nthree\nTODO 🦀"), vec![(2, "// TODO: \"ship\"".into()), (4, "TODO 🦀".into())]);
        assert!(todo_lines("clean\n").is_empty());
    }
    #[test]
    fn json_escape_handles_quotes_backslashes_and_controls() {
        assert_eq!(json_escape("a\"\\\t\n"), "a\\\"\\\\\\t\\n");
    }
}
