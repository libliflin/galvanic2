//! galvanic library — pipeline phases exposed as public modules.
//!
//! Each module corresponds to one stage of the compilation pipeline:
//!
//! ```text
//! source text
//!     → lexer::tokenize()   → Vec<Token>       [src/lexer.rs]
//!     → parser::parse()     → SourceFile (AST) [src/parser.rs, src/ast.rs]
//!     → lower::lower()      → Module (IR)      [src/lower.rs, src/ir.rs]
//!     → codegen::emit_asm() → String (GAS)     [src/codegen.rs]
//! ```
//!
//! See `.lathe/skills/architecture.md` for the full target shape.

pub mod lexer;
