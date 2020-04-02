use std::{env, fs, io::{self, Read}};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--help") { println!("usage: is-this-rust-yet [FILE|-]"); return; }
    if args.len() > 1 || args.first().is_some_and(|a| a.starts_with('-') && a != "-") { eprintln!("error: expected one file or -"); std::process::exit(2); }
    let mut source = String::new();
    if args.first().map(String::as_str) == Some("-") || args.is_empty() {
        if let Err(e) = io::stdin().read_to_string(&mut source) { eprintln!("error: {e}"); std::process::exit(2); }
    } else {
        match fs::read_to_string(&args[0]) { Ok(s) => source = s, Err(e) => { eprintln!("error: {e}"); std::process::exit(2); } }
    }
    inspect(&source);
}

fn inspect(source: &str) {
    let lines = source.lines().count();
    let functions = source.matches("fn ").count();
    let tests = source.matches("#[test]").count();
    let unsafe_blocks = source.matches("unsafe").count();
    let unwraps = source.matches("unwrap").count();
    let todos = source.matches("TODO").count();
    println!("Is this Rust yet? tiny source-text inspection\n");
    println!("bytes: {}\nlines: {lines}\nfunctions: {functions}\ntests: {tests}\nunsafe mentions: {unsafe_blocks}\nunwrap mentions: {unwraps}\nTODOs: {todos}", source.len());
    println!("\nchecklist:");
    println!("  {} fn keyword(s): the compiler may recognize a function", if functions > 0 { "[x]" } else { "[ ]" });
    println!("  {} test attribute(s): future-you has backup", if tests > 0 { "[x]" } else { "[ ]" });
    println!("  {} TODO(s): the 2020 lockdown backlog lives", if todos > 0 { "[x]" } else { "[ ]" });
    println!("\nverdict: {}", if functions > 0 { "Rust-shaped. Please ask rustc before ordering a commemorative hoodie." } else { "Rust-adjacent. It may be a README wearing a trench coat." });
}
