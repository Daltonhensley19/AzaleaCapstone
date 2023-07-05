//! Defines the tokens that are supported by the Morehead Lambda Compiler. 
//!
//! To add support for a new `Token`, you must first add it to this file.

use std::collections::BTreeSet;

use crate::span::SpanPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Let,
    Ident,
    Reserved,
    TQualifer,
    Semicolon,
    Assign,
    Plus,
    Minus,
    Div,
    Mul,
    NumLit,
    BoolLit,
    FloatLit,
    Main,
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    RecordDot,
    ExRange,
    LParn,
    RParn,
    LBracket,
    RBracket,
    Sep,
    FnDef,
    RetArrow,
}

impl TokenKind {
    fn is_bool_literal<P: AsRef<str>>(raw_token_content: P) -> bool {
        match raw_token_content.as_ref()
        {
            "true" | "false" => true,
            _ => false,
        }
    }

    fn is_reserved<P: AsRef<str>>(raw_token_content: P) -> bool {
        let mut reserved = BTreeSet::from([
            "let", "main", "float", "int", "text", "record", "enum", "as",
        ]);

        reserved.contains(&raw_token_content.as_ref())
    }

    pub fn refined_or_ident<P: AsRef<str>>(raw_token_content: P) -> TokenKind {
        if TokenKind::is_bool_literal(&raw_token_content)
        {
            return TokenKind::BoolLit;
        }

        if TokenKind::is_reserved(raw_token_content)
        {
            return TokenKind::Reserved;
        }

        TokenKind::Ident
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum TokenHint {
    #[default]
    Undetermined,
    IdentOrKeyword,
    Number,
}

#[derive(Debug)]
pub struct Token {
    raw_content: String,
    kind: TokenKind,
    span_start: SpanPoint,
    span_end: SpanPoint,
}

impl Token {
    pub fn new(
        raw_content: String,
        kind: TokenKind,
        span_start: SpanPoint,
        span_end: SpanPoint,
    ) -> Self {
        Self {
            raw_content,
            kind,
            span_start,
            span_end,
        }
    }

    pub fn span_end_mut(&mut self) -> &mut SpanPoint {
        &mut self.span_end
    }

    pub fn span_start_mut(&mut self) -> &mut SpanPoint {
        &mut self.span_start
    }

    pub fn span_end_ref(&self) -> &SpanPoint {
        &self.span_end
    }

    pub fn span_start_ref(&self) -> &SpanPoint {
        &self.span_start
    }

    pub fn get_token_kind(&self) -> TokenKind {
        self.kind
    }

    pub fn is_a(&self, kind: TokenKind) -> bool {
        self.kind == kind
    }
}
