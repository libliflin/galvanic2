//! Abstract Syntax Tree — FLS §3 Items, §6 Expressions, §8 Statements, §9 Functions
//!
//! This AST covers the minimal surface needed to represent programs accepted
//! by the current parser milestone. Each type is documented with its FLS
//! section and a cache-line note.

/// A parsed source file. FLS §18: a crate is the unit of compilation.
///
/// Cache-line note: `SourceFile` is heap-allocated and visited once per
/// compilation; cache pressure is not a concern for this type.
#[derive(Debug)]
pub struct SourceFile {
    /// Top-level items in declaration order.
    pub items: Vec<Item>,
}

/// A top-level item. FLS §3.
///
/// Cache-line note: `Item` holds a `Box<_>` pointer (8 bytes), so a
/// `Vec<Item>` stores one pointer per slot — 8 fit per cache line.
#[derive(Debug)]
pub enum Item {
    /// A function definition. FLS §9.
    Fn(Box<FnDef>),
}

/// A function definition. FLS §9.
///
/// FLS §9:1–5: A function item defines a name, parameter list, optional
/// return type, and a body block.
///
/// Cache-line note: name (4) + params Vec (24) + ret (2) + body (48) ≈ 80
/// bytes. `FnDef` is always behind a `Box` so the enum stays pointer-sized.
#[derive(Debug)]
pub struct FnDef {
    /// Function name as a packed lexer span.
    /// Use `lexer::token_text(name, source)` to recover the text.
    ///
    /// FLS §9:3: the function name is an identifier.
    pub name: u32,

    /// Function parameters. FLS §9:8–12.
    /// Empty for milestone 2; extended in future milestones.
    pub params: Vec<Param>,

    /// Return type. `None` means the unit type `()`. FLS §9:15–20.
    pub ret: Option<Ty>,

    /// Function body. FLS §9:22.
    pub body: Block,
}

/// A function parameter. FLS §9:8–12.
#[derive(Debug)]
pub struct Param {
    /// Parameter name (packed lexer span).
    pub name: u32,
    /// Parameter type.
    pub ty: Ty,
}

/// A type. FLS §4. Only scalar primitives for milestone 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    /// `i32` — FLS §4.3: signed 32-bit integer.
    I32,
}

/// A block expression (function body or braced block). FLS §6.4.
///
/// Cache-line note: two `Vec` headers (24 bytes each) + one `Option<Box<Expr>>`
/// (16 bytes) = 64 bytes. One `Block` fits exactly in a cache line.
#[derive(Debug)]
pub struct Block {
    /// Statements in execution order. FLS §8.
    pub stmts: Vec<Stmt>,
    /// Optional tail expression (no trailing semicolon). FLS §6.4:8.
    pub tail: Option<Box<Expr>>,
}

/// A statement. FLS §8.
#[derive(Debug)]
pub enum Stmt {
    /// An expression statement: expression followed by `;`. FLS §8.2.
    Expr(Box<Expr>),
}

/// An expression. FLS §6. Only the forms needed for milestone 2.
#[derive(Debug)]
pub enum Expr {
    /// An integer literal. FLS §6.2, §2.4.4.1.
    /// The `u32` is a packed lexer span; use `lexer::token_text` to recover text.
    IntLit(u32),

    /// A boolean literal. FLS §6.2, §2.4.7.
    BoolLit(bool),

    /// A `return` expression. FLS §6.19.
    /// The inner expression is `None` for a bare `return`.
    Return(Option<Box<Expr>>),
}
