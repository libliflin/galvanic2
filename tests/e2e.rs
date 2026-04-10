//! End-to-end tests.
//!
//! Each future milestone here compiles a small Rust program through the
//! galvanic pipeline (lex → parse → lower → codegen → assemble → link) and
//! verifies the resulting ARM64 binary's runtime behavior — both exit code
//! AND emitted instructions, per the assembly inspection pattern.
//!
//! Stage 0: a single placeholder so `cargo test --test e2e` succeeds. The
//! first real milestone test will replace it. See `.lathe/skills/testing.md`
//! for the expected pattern.

#[test]
fn placeholder() {
    // Placeholder. Remove when the first real milestone test lands.
}
