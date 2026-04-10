//! galvanic — clean-room ARM64 Rust compiler from the Ferrocene Language
//! Specification, with cache-line alignment as a first-class design constraint.
//!
//! Stage 0: CLI shell. Reads a source file from `argv[1]` and exits cleanly.
//! The lex → parse → lower → codegen → assemble → link pipeline will be filled
//! in milestone by milestone, each anchored to a specific `FLS §X.Y` citation.
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
    if let Err(e) = fs::read_to_string(source_path) {
        eprintln!("galvanic: cannot read {source_path}: {e}");
        return ExitCode::FAILURE;
    }

    // TODO(milestone 1): lex → parse → lower → codegen → assemble → link.
    ExitCode::SUCCESS
}
