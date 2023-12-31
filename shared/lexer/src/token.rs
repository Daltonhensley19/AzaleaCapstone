//! Defines the tokens that are supported by the Morehead Azalea Compiler.
//!
//! To add support for a new `Token`, you must first add it to this file.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::span::SpanPoint;
use serde::Serialize;
    
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Ident,
    IntTy,
    FloatTy,
    TextTy,
    BoolTy,
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
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    NEq,
    Not,
    RecordDot,
    ExRange,
    LParn,
    RParn,
    LBracket,
    RBracket,
    LSBracket,
    RSBracket,
    Sep,
    FnDef,
    RetArrow,
    StructKw,
    ChoiceKw,
    MainKw,
    AsKw,
    AndKw,
    OrKw,
    InKw,
    LetKw,
    IfKw,
    ElifKw,
    ElseKw,
    ForKw,
    WhileKw,
    EOF

}

impl From<TokenKind> for &str {
    fn from(value: TokenKind) -> &'static str {
        match value
        {
            TokenKind::Ident => "ident",
            TokenKind::TQualifer => "::",
            TokenKind::Semicolon => ";",
            TokenKind::Assign => "<-",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Div => "/",
            TokenKind::Mul => "*",
            TokenKind::NumLit => "number literal",
            TokenKind::BoolLit => "bool literal",
            TokenKind::FloatLit => "float literal",
            TokenKind::Lt => "<",
            TokenKind::Lte => "<=",
            TokenKind::Gt => ">",
            TokenKind::Gte => ">=",
            TokenKind::Eq => "==",
            TokenKind::RecordDot => ".",
            TokenKind::ExRange => "..",
            TokenKind::LParn => "(",
            TokenKind::RParn => ")",
            TokenKind::LBracket => "{",
            TokenKind::RBracket => "}",
	    TokenKind::LSBracket => "[",
            TokenKind::RSBracket => "]",
            TokenKind::Sep => ",",
            TokenKind::FnDef => "=",
            TokenKind::RetArrow => "->",
            TokenKind::IntTy => "int",
            TokenKind::FloatTy => "float",
            TokenKind::TextTy => "text",
            TokenKind::BoolTy => "bool",
            TokenKind::StructKw => "structure",
            TokenKind::ChoiceKw => "choice",
            TokenKind::MainKw => "main",
            TokenKind::AsKw => "as",
            TokenKind::AndKw => "and",
            TokenKind::OrKw => "or",
            TokenKind::InKw => "in",
            TokenKind::LetKw => "let",
            TokenKind::IfKw => "if",
            TokenKind::ElifKw => "elif",
            TokenKind::ElseKw => "else",
            TokenKind::WhileKw => "while",
            TokenKind::ForKw => "for",
	    TokenKind::NEq => "!=",
	    TokenKind::Not => "!",
            TokenKind::EOF => "$$"
        }
    }
}

impl TokenKind {
    // Helper to generically check if
    fn is_reserved<P: AsRef<str>>(raw_token_content: P) -> Option<TokenKind> {
        let mut reserved = BTreeMap::from([
            ("float", TokenKind::FloatTy),
            ("int", TokenKind::IntTy),
            ("text", TokenKind::TextTy),
            ("bool", TokenKind::BoolTy),
            ("structure", TokenKind::StructKw),
            ("choice", TokenKind::ChoiceKw),
            ("main", TokenKind::MainKw),
            ("as", TokenKind::AsKw),
            ("or", TokenKind::OrKw),
            ("and", TokenKind::AndKw),
            ("in", TokenKind::InKw),
            ("let", TokenKind::LetKw),
            ("if", TokenKind::IfKw),
            ("elif", TokenKind::ElifKw),
            ("else", TokenKind::ElseKw),
            ("while", TokenKind::WhileKw),
            ("for", TokenKind::ForKw),
            ("true", TokenKind::BoolLit),
            ("false", TokenKind::BoolLit),

        ]);

        let tok_kind = reserved.get(&raw_token_content.as_ref());

        tok_kind.copied()
    }

    // Helper to refine ident to bool literal.
    // This is private, so it is many used internally.
    fn is_bool_literal<P: AsRef<str>>(raw_token_content: P) -> bool {
        matches!(raw_token_content.as_ref(), "true" | "false")
    }

    // Returns TokenKind and bool if the TokenKind is reserved
    pub fn refined_or_ident<P: AsRef<str>>(raw_token_content: P) -> (TokenKind, bool) {
        // Return early if we can refine ident to a reserved token_kind
        if let Some(token_kind) = TokenKind::is_reserved(&raw_token_content)
        {
            let is_reserved = true;
            return (token_kind, is_reserved);  
        }

        // Otherwise, refinement failed so return ident token_kind
        let is_reserved = false;
        (TokenKind::Ident, is_reserved)
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum TokenHint {
    #[default]
    Undetermined,
    IdentOrKeyword,
    Number,
}

#[derive(Serialize, Debug, Clone)]
pub struct Token {
    raw_content: String,
    #[serde(skip)]
    kind: TokenKind,
    #[serde(skip)]
    span_start: SpanPoint,
    #[serde(skip)]
    span_end: SpanPoint,
    #[serde(skip)]
    file_index: usize,
    #[serde(skip)]
    reserved: bool,
}

// Way to print a `Token` using println!()
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "{}", self.raw_content)
    }
}


impl Token {
    pub fn new(
        raw_content: String,
        kind: TokenKind,
        span_start: SpanPoint,
        span_end: SpanPoint,
        file_index: usize,
        reserved: bool,
    ) -> Self {
        Self {
            raw_content,
            kind,
            span_start,
            span_end,
            file_index,
            reserved,
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

    pub fn get_file_index(&self) -> usize {
        self.file_index
    }
    pub fn get_raw_content(&self) -> &str {
        &self.raw_content
    }
}
