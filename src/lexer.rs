//! Lexer — FLS §2 Lexical Elements
//!
//! Converts a source string into a flat sequence of `Token` values.
//! The entry point is [`tokenize`].
//!
//! ## Cache-line design
//!
//! `Token` is `repr(u8)` with a 1-byte discriminant and at most a `u32`
//! payload, giving a total size of **8 bytes**.  Eight tokens fit per
//! 64-byte cache line, keeping hot tokenizer loops cache-friendly.
//!
//! Span encoding for `Ident`, `IntLit`, and `FloatLit`: the `u32` packs
//! `bits[31:8]` = start byte offset (max 16 MiB source) and
//! `bits[7:0]` = byte length (max 255 bytes per token).  Use
//! [`span_start`], [`span_len`], and [`token_text`] to unpack.

/// A lexical token produced by [`tokenize`].
///
/// Cache-line note: 8 bytes (discriminant u8 + 3-byte pad + u32 payload).
/// Eight tokens fit per 64-byte cache line.
///
/// FLS §2.2: Lexical Elements, Separators, and Punctuation
/// FLS §2.3: Identifiers
/// FLS §2.4: Literals
/// FLS §2.6: Keywords
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
    // ── FLS §2.6.1 Strict keywords ───────────────────────────────────────────
    Fn,
    Return,
    Let,
    Mut,
    If,
    Else,
    While,
    For,
    In,
    /// FLS §2.4.7
    True,
    /// FLS §2.4.7
    False,

    // ── Punctuation and operators — no payload ────────────────────────────────
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `;`
    Semicolon,
    /// `:`
    Colon,
    /// `::`
    ColonColon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `->`
    Arrow,
    /// `=>`
    FatArrow,
    /// `=`
    Eq,
    /// `==`
    EqEq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `&`
    Amp,
    /// `&&`
    AmpAmp,
    /// `|`
    Pipe,
    /// `||`
    PipePipe,
    /// `^`
    Caret,
    /// `!`
    Bang,

    // ── Tokens with packed span: bits[31:8] = start, bits[7:0] = len ─────────
    /// An identifier or path segment. FLS §2.3
    Ident(u32),
    /// An integer literal. FLS §2.4.4.1
    IntLit(u32),
    /// A floating-point literal. FLS §2.4.4.2
    FloatLit(u32),

    /// End of input.
    Eof,
}

// Structural claim: Token is exactly 8 bytes.
// Eight tokens fit per 64-byte cache line.
const _TOKEN_SIZE: () = assert!(
    core::mem::size_of::<Token>() == 8,
    "Token must be 8 bytes to fit 8 per cache line"
);

/// Pack a byte offset and byte length into the span `u32` used by
/// `Token::Ident`, `Token::IntLit`, and `Token::FloatLit`.
///
/// Panics in debug builds if `start >= 2^24` or `len >= 256`.
#[inline]
pub fn pack_span(start: u32, len: u32) -> u32 {
    debug_assert!(start < (1 << 24), "source offset exceeds 16 MiB");
    debug_assert!(len < 256, "token text exceeds 255 bytes");
    (start << 8) | len
}

/// Extract the start byte offset from a packed span.
#[inline]
pub fn span_start(packed: u32) -> usize {
    (packed >> 8) as usize
}

/// Extract the byte length from a packed span.
#[inline]
pub fn span_len(packed: u32) -> usize {
    (packed & 0xFF) as usize
}

/// Return the source slice corresponding to a packed span.
#[inline]
pub fn token_text(packed: u32, source: &str) -> &str {
    let start = span_start(packed);
    let len = span_len(packed);
    &source[start..start + len]
}

/// Lex `source` into a flat token sequence.
///
/// The returned `Vec` always ends with `Token::Eof`.  Unrecognised
/// characters are silently skipped (error reporting is deferred to a
/// later milestone).
///
/// FLS §2: Lexical Elements
pub fn tokenize(source: &str) -> Vec<Token> {
    let src = source.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < src.len() {
        // FLS §2.2: whitespace is not significant between tokens.
        if src[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }

        // FLS §2.5: line comments begin with `//` and extend to end of line.
        if i + 1 < src.len() && src[i] == b'/' && src[i + 1] == b'/' {
            while i < src.len() && src[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // FLS §2.3 / §2.6: identifiers and keywords start with a letter or `_`.
        if src[i].is_ascii_alphabetic() || src[i] == b'_' {
            let start = i;
            while i < src.len() && (src[i].is_ascii_alphanumeric() || src[i] == b'_') {
                i += 1;
            }
            let text = &source[start..i];
            let span = pack_span(start as u32, (i - start) as u32);
            let tok = match text {
                "fn" => Token::Fn,
                "return" => Token::Return,
                "let" => Token::Let,
                "mut" => Token::Mut,
                "if" => Token::If,
                "else" => Token::Else,
                "while" => Token::While,
                "for" => Token::For,
                "in" => Token::In,
                "true" => Token::True,
                "false" => Token::False,
                _ => Token::Ident(span),
            };
            tokens.push(tok);
            continue;
        }

        // FLS §2.4.4.1: integer literals (decimal only for milestone 1).
        if src[i].is_ascii_digit() {
            let start = i;
            while i < src.len() && src[i].is_ascii_digit() {
                i += 1;
            }
            let span = pack_span(start as u32, (i - start) as u32);
            tokens.push(Token::IntLit(span));
            continue;
        }

        // Operators and punctuation — longest-match where ambiguous.
        let tok = match src[i] {
            b'(' => {
                i += 1;
                Token::LParen
            }
            b')' => {
                i += 1;
                Token::RParen
            }
            b'{' => {
                i += 1;
                Token::LBrace
            }
            b'}' => {
                i += 1;
                Token::RBrace
            }
            b'[' => {
                i += 1;
                Token::LBracket
            }
            b']' => {
                i += 1;
                Token::RBracket
            }
            b';' => {
                i += 1;
                Token::Semicolon
            }
            b',' => {
                i += 1;
                Token::Comma
            }
            b'.' => {
                i += 1;
                Token::Dot
            }
            b'^' => {
                i += 1;
                Token::Caret
            }
            b'%' => {
                i += 1;
                Token::Percent
            }
            b'*' => {
                i += 1;
                Token::Star
            }
            b'+' => {
                i += 1;
                Token::Plus
            }
            b'/' => {
                i += 1;
                Token::Slash
            }
            b'-' => {
                if i + 1 < src.len() && src[i + 1] == b'>' {
                    i += 2;
                    Token::Arrow
                } else {
                    i += 1;
                    Token::Minus
                }
            }
            b'=' => {
                if i + 1 < src.len() && src[i + 1] == b'>' {
                    i += 2;
                    Token::FatArrow
                } else if i + 1 < src.len() && src[i + 1] == b'=' {
                    i += 2;
                    Token::EqEq
                } else {
                    i += 1;
                    Token::Eq
                }
            }
            b'!' => {
                if i + 1 < src.len() && src[i + 1] == b'=' {
                    i += 2;
                    Token::Ne
                } else {
                    i += 1;
                    Token::Bang
                }
            }
            b'<' => {
                if i + 1 < src.len() && src[i + 1] == b'=' {
                    i += 2;
                    Token::Le
                } else {
                    i += 1;
                    Token::Lt
                }
            }
            b'>' => {
                if i + 1 < src.len() && src[i + 1] == b'=' {
                    i += 2;
                    Token::Ge
                } else {
                    i += 1;
                    Token::Gt
                }
            }
            b'&' => {
                if i + 1 < src.len() && src[i + 1] == b'&' {
                    i += 2;
                    Token::AmpAmp
                } else {
                    i += 1;
                    Token::Amp
                }
            }
            b'|' => {
                if i + 1 < src.len() && src[i + 1] == b'|' {
                    i += 2;
                    Token::PipePipe
                } else {
                    i += 1;
                    Token::Pipe
                }
            }
            b':' => {
                if i + 1 < src.len() && src[i + 1] == b':' {
                    i += 2;
                    Token::ColonColon
                } else {
                    i += 1;
                    Token::Colon
                }
            }
            _ => {
                // Unknown character — skip (error reporting deferred).
                i += 1;
                continue;
            }
        };
        tokens.push(tok);
    }

    tokens.push(Token::Eof);
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    // Structural claim: Token is 8 bytes — fits 8 per 64-byte cache line.
    // This mirrors the const assert above; the test gives a clear failure
    // message if someone adds a variant that bloats the type.
    #[test]
    fn token_is_eight_bytes() {
        assert_eq!(
            core::mem::size_of::<Token>(),
            8,
            "Token grew past 8 bytes — fix the variant or update the cache-line claim"
        );
    }

    #[test]
    fn fn_main_returns_i32() {
        // FLS §9: function item with return type annotation.
        // "fn main() -> i32 { 42 }"
        //  0123456789012345678901234
        //            1111111111222222
        let src = "fn main() -> i32 { 42 }";
        let tokens = tokenize(src);
        assert_eq!(tokens[0], Token::Fn);
        assert_eq!(tokens[1], Token::Ident(pack_span(3, 4))); // "main" at 3..7
        assert_eq!(tokens[2], Token::LParen);
        assert_eq!(tokens[3], Token::RParen);
        assert_eq!(tokens[4], Token::Arrow);
        assert_eq!(tokens[5], Token::Ident(pack_span(13, 3))); // "i32" at 13..16
        assert_eq!(tokens[6], Token::LBrace);
        assert_eq!(tokens[7], Token::IntLit(pack_span(19, 2))); // "42" at 19..21
        assert_eq!(tokens[8], Token::RBrace);
        assert_eq!(tokens[9], Token::Eof);
    }

    #[test]
    fn keyword_recognition() {
        // FLS §2.6.1: strict keywords must not be lexed as identifiers.
        let tokens = tokenize("fn let mut if else while for in true false return");
        assert_eq!(tokens[0], Token::Fn);
        assert_eq!(tokens[1], Token::Let);
        assert_eq!(tokens[2], Token::Mut);
        assert_eq!(tokens[3], Token::If);
        assert_eq!(tokens[4], Token::Else);
        assert_eq!(tokens[5], Token::While);
        assert_eq!(tokens[6], Token::For);
        assert_eq!(tokens[7], Token::In);
        assert_eq!(tokens[8], Token::True);
        assert_eq!(tokens[9], Token::False);
        assert_eq!(tokens[10], Token::Return);
        assert_eq!(tokens[11], Token::Eof);
    }

    #[test]
    fn keyword_prefix_is_ident() {
        // FLS §2.3: "function" starts with "fn" but is an identifier, not a keyword.
        let tokens = tokenize("function");
        assert_eq!(tokens[0], Token::Ident(pack_span(0, 8)));
        assert_eq!(tokens[1], Token::Eof);
    }

    #[test]
    fn token_text_extraction() {
        // FLS §2.3: identifier text is recoverable from the packed span.
        let src = "hello_world";
        let tokens = tokenize(src);
        match tokens[0] {
            Token::Ident(span) => {
                assert_eq!(token_text(span, src), "hello_world");
            }
            other => panic!("expected Ident, got {other:?}"),
        }
    }

    #[test]
    fn line_comment_skipped() {
        // FLS §2.5: line comments are not tokens.
        // "fn // this is a comment\nmain"
        //  01234567890123456789012345678
        //            1111111111222222222
        // "main" starts at offset 24.
        let src = "fn // this is a comment\nmain";
        let tokens = tokenize(src);
        assert_eq!(tokens[0], Token::Fn);
        assert_eq!(tokens[1], Token::Ident(pack_span(24, 4)));
        assert_eq!(tokens[2], Token::Eof);
    }

    #[test]
    fn two_char_operators() {
        // FLS §2.2: two-character operators use longest-match.
        let tokens = tokenize("-> => :: == != <= >= && ||");
        assert_eq!(tokens[0], Token::Arrow);
        assert_eq!(tokens[1], Token::FatArrow);
        assert_eq!(tokens[2], Token::ColonColon);
        assert_eq!(tokens[3], Token::EqEq);
        assert_eq!(tokens[4], Token::Ne);
        assert_eq!(tokens[5], Token::Le);
        assert_eq!(tokens[6], Token::Ge);
        assert_eq!(tokens[7], Token::AmpAmp);
        assert_eq!(tokens[8], Token::PipePipe);
        assert_eq!(tokens[9], Token::Eof);
    }

    #[test]
    fn empty_input_yields_only_eof() {
        let tokens = tokenize("");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Eof);
    }
}
