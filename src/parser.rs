//! Parser — FLS §3 Items, §6 Expressions, §8 Statements, §9 Functions
//!
//! Converts a flat `Vec<Token>` from the lexer into a [`SourceFile`] AST.
//! The entry point is [`parse`].
//!
//! Error reporting is deferred (FLS §16). Unexpected tokens panic in debug
//! mode (compile-time protection against misuse) and produce a best-effort
//! partial parse in release mode via token-skipping at the item boundary.

use crate::ast::{Block, Expr, FnDef, Item, Param, SourceFile, Stmt, Ty};
use crate::lexer::{span_len, span_start, Token};

/// Parse `tokens` into a [`SourceFile`] AST.
///
/// `source` is the original source text, needed to resolve identifier spans
/// to type names (e.g. `"i32"`).
///
/// FLS §18: A source file contains a sequence of items. Items are parsed
/// in declaration order and collected into [`SourceFile::items`].
pub fn parse(tokens: &[Token], source: &str) -> SourceFile {
    let mut p = Parser { tokens, source, pos: 0 };
    p.parse_source_file()
}

// ── Internal parser state ────────────────────────────────────────────────────

struct Parser<'src> {
    tokens: &'src [Token],
    source: &'src str,
    pos: usize,
}

impl<'src> Parser<'src> {
    // ── Token stream primitives ──────────────────────────────────────────────

    /// Return the current token without advancing. Always safe: the lexer
    /// guarantees the stream ends with `Token::Eof`.
    fn peek(&self) -> Token {
        self.tokens[self.pos]
    }

    /// Consume and return the current token.
    fn bump(&mut self) -> Token {
        let t = self.tokens[self.pos];
        // Stay at the last position (Eof) rather than going out of bounds.
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    /// Return `true` if the current token equals `t` (for no-payload tokens).
    fn at(&self, t: Token) -> bool {
        self.peek() == t
    }

    /// Consume the current token, panicking in debug builds if it is not `t`.
    fn expect(&mut self, t: Token) {
        debug_assert_eq!(
            self.peek(),
            t,
            "parse error: expected {t:?}, found {:?} (pos {})",
            self.peek(),
            self.pos
        );
        self.bump();
    }

    // ── Grammar rules ────────────────────────────────────────────────────────

    /// Parse the top-level item list. FLS §18.
    fn parse_source_file(&mut self) -> SourceFile {
        let mut items = Vec::new();
        while !self.at(Token::Eof) {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                // Unknown token at item position — skip (error reporting deferred).
                self.bump();
            }
        }
        SourceFile { items }
    }

    /// Try to parse one item. Returns `None` if the current token does not
    /// begin a known item form. FLS §3.
    fn parse_item(&mut self) -> Option<Item> {
        match self.peek() {
            Token::Fn => Some(Item::Fn(Box::new(self.parse_fn()))),
            _ => None,
        }
    }

    /// Parse a function definition. FLS §9.
    ///
    /// Grammar: `fn` IDENT `(` params `)` (`->` type)? block
    fn parse_fn(&mut self) -> FnDef {
        // FLS §9:1: the `fn` keyword opens the definition.
        self.expect(Token::Fn);

        // FLS §9:3: function name is an identifier.
        let name = match self.bump() {
            Token::Ident(span) => span,
            other => panic!("parse error: expected function name, found {other:?}"),
        };

        // FLS §9:8–12: parameter list enclosed in parentheses.
        self.expect(Token::LParen);
        let params = self.parse_params();
        self.expect(Token::RParen);

        // FLS §9:15–20: optional return type annotation.
        let ret = if self.at(Token::Arrow) {
            self.bump(); // consume `->`
            Some(self.parse_ty())
        } else {
            None // absent annotation → return type is unit `()`. FLS §9:20.
        };

        // FLS §9:22: function body is a block expression.
        let body = self.parse_block();

        FnDef { name, params, ret, body }
    }

    /// Parse a comma-separated parameter list (between `(` and `)`). FLS §9:8–12.
    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        while !self.at(Token::RParen) && !self.at(Token::Eof) {
            // FLS §9:10: each parameter is `pat : Type`.
            let name = match self.bump() {
                Token::Ident(span) => span,
                other => panic!("parse error: expected parameter name, found {other:?}"),
            };
            self.expect(Token::Colon);
            let ty = self.parse_ty();
            params.push(Param { name, ty });
            if self.at(Token::Comma) {
                self.bump();
            }
        }
        params
    }

    /// Parse a type. FLS §4. Only scalar primitives for milestone 2.
    fn parse_ty(&mut self) -> Ty {
        match self.bump() {
            Token::Ident(span) => {
                let start = span_start(span);
                let end = start + span_len(span);
                match &self.source[start..end] {
                    // FLS §4.3: numeric types — signed integers.
                    "i32" => Ty::I32,
                    other => panic!("parse error: unknown type `{other}`"),
                }
            }
            other => panic!("parse error: expected type, found {other:?}"),
        }
    }

    /// Parse a block expression. FLS §6.4.
    ///
    /// A block is a `{` sequence of statements and an optional tail expression `}`.
    /// FLS §6.4:8: if the last item in the block has no `;`, it is the tail
    /// expression and determines the block's type and value.
    fn parse_block(&mut self) -> Block {
        self.expect(Token::LBrace);
        let mut stmts = Vec::new();
        let mut tail = None;

        while !self.at(Token::RBrace) && !self.at(Token::Eof) {
            let expr = self.parse_expr();
            if self.at(Token::Semicolon) {
                // FLS §8.2: expression statement — expression followed by `;`.
                self.bump(); // consume `;`
                stmts.push(Stmt::Expr(Box::new(expr)));
            } else {
                // FLS §6.4:8: tail expression (no `;`) determines the block value.
                tail = Some(Box::new(expr));
                break;
            }
        }

        self.expect(Token::RBrace);
        Block { stmts, tail }
    }

    /// Parse an expression. FLS §6. Only the forms needed for milestone 2.
    fn parse_expr(&mut self) -> Expr {
        match self.peek() {
            // FLS §6.19: return expression.
            Token::Return => {
                self.bump(); // consume `return`
                // A `return` with no value returns unit. FLS §6.19:5.
                if self.at(Token::Semicolon)
                    || self.at(Token::RBrace)
                    || self.at(Token::Eof)
                {
                    Expr::Return(None)
                } else {
                    Expr::Return(Some(Box::new(self.parse_expr())))
                }
            }
            // FLS §6.2, §2.4.4.1: integer literal expression.
            Token::IntLit(span) => {
                self.bump();
                Expr::IntLit(span)
            }
            // FLS §6.2, §2.4.7: boolean literal expressions.
            Token::True => {
                self.bump();
                Expr::BoolLit(true)
            }
            Token::False => {
                self.bump();
                Expr::BoolLit(false)
            }
            other => panic!("parse error: unexpected token {other:?}"),
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Item, Ty};
    use crate::lexer::{token_text, tokenize};

    /// Milestone 2: parse `fn main() -> i32 { 42 }` — tail expression body.
    ///
    /// FLS §9: function with return-type annotation.
    /// FLS §6.4:8: a block's tail expression (no `;`) is the block value.
    /// FLS §6.2: integer literal as an expression.
    #[test]
    fn parse_fn_main_tail_expr() {
        let src = "fn main() -> i32 { 42 }";
        let tokens = tokenize(src);
        let sf = parse(&tokens, src);

        assert_eq!(sf.items.len(), 1);
        let Item::Fn(f) = &sf.items[0];

        assert_eq!(token_text(f.name, src), "main");
        assert_eq!(f.ret, Some(Ty::I32));
        assert!(f.params.is_empty());
        assert!(f.body.stmts.is_empty());

        match f.body.tail.as_deref() {
            Some(Expr::IntLit(span)) => {
                assert_eq!(token_text(*span, src), "42");
            }
            other => panic!("expected IntLit tail expression, got {other:?}"),
        }
    }

    /// Milestone 2: parse `fn main() -> i32 { return 42; }` — return statement.
    ///
    /// FLS §9: function definition.
    /// FLS §6.19: return expression carries a value.
    /// FLS §8.2: expression statement (expression + `;`).
    #[test]
    fn parse_fn_main_return_stmt() {
        let src = "fn main() -> i32 { return 42; }";
        let tokens = tokenize(src);
        let sf = parse(&tokens, src);

        assert_eq!(sf.items.len(), 1);
        let Item::Fn(f) = &sf.items[0];

        assert_eq!(f.body.stmts.len(), 1);
        assert!(f.body.tail.is_none(), "return stmt must not become tail expr");

        let Stmt::Expr(inner) = &f.body.stmts[0];
        match inner.as_ref() {
            Expr::Return(Some(val)) => match val.as_ref() {
                Expr::IntLit(span) => assert_eq!(token_text(*span, src), "42"),
                other => panic!("expected IntLit inside return, got {other:?}"),
            },
            other => panic!("expected Return expression, got {other:?}"),
        }
    }

    /// Milestone 2: parse `fn noop() {}` — no return type, empty body.
    ///
    /// FLS §9:20: absent return-type annotation → return type is unit `()`.
    /// FLS §6.4:8: an empty block has no tail expression and evaluates to `()`.
    #[test]
    fn parse_fn_no_ret_empty_body() {
        let src = "fn noop() {}";
        let tokens = tokenize(src);
        let sf = parse(&tokens, src);

        assert_eq!(sf.items.len(), 1);
        let Item::Fn(f) = &sf.items[0];

        assert_eq!(token_text(f.name, src), "noop");
        assert_eq!(f.ret, None, "absent annotation should parse as None (unit)");
        assert!(f.params.is_empty());
        assert!(f.body.stmts.is_empty());
        assert!(f.body.tail.is_none());
    }

    /// Milestone 2: parse a function with a typed parameter. FLS §9:8–12.
    ///
    /// FLS §9:10: each parameter is `identifier : Type`.
    /// Body is `0` (not the parameter) because `Expr::Ident` is deferred
    /// to the milestone that introduces identifier expressions.
    #[test]
    fn parse_fn_with_param() {
        let src = "fn id(x: i32) -> i32 { 0 }";
        let tokens = tokenize(src);
        let sf = parse(&tokens, src);

        assert_eq!(sf.items.len(), 1);
        let Item::Fn(f) = &sf.items[0];

        assert_eq!(f.params.len(), 1);
        assert_eq!(token_text(f.params[0].name, src), "x");
        assert_eq!(f.params[0].ty, Ty::I32);
    }
}
