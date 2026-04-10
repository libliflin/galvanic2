//! galvanic — clean-room ARM64 Rust compiler from the Ferrocene Language
//! Specification, with cache-line alignment as a first-class design constraint.
//!
//! The lex → parse → lower → codegen → assemble → link pipeline is filled in
//! milestone by milestone, each anchored to a specific `FLS §X.Y` citation.
//! See `.lathe/skills/architecture.md` for the target shape.

use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: galvanic <source.rs>");
        return ExitCode::FAILURE;
    }

    let source_path = &args[1];
    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("galvanic: cannot read {source_path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    // FLS §2: lex the source into tokens.
    let tokens = galvanic::lexer::tokenize(&source);
    eprintln!(
        "galvanic: lexed {} token(s) from {source_path}",
        tokens.len().saturating_sub(1) // exclude Eof
    );

    // TODO(milestone 2): parse → lower → codegen → assemble → link.
    ExitCode::SUCCESS
}
