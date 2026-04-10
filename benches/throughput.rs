//! Throughput benchmarks.
//!
//! Stage 0: empty stub. The `[[bench]]` entry in `Cargo.toml` declares this
//! file with `harness = false`, so cargo only requires a `fn main()` here —
//! it does not require criterion at this stage. Real bench cases will be
//! added when there is a lexer / parser / codegen pipeline to measure
//! against. Until then this is the smallest valid bench target that lets
//! `cargo build` parse the manifest.

fn main() {}
