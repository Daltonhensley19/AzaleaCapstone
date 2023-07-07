//! AST parser for the Morehead Lambda Compiler
//!
//! NOTE: The formal grammar is defined in the `grammar/` directory inside the file
//! `formal_grammar.pest`.

use std::{cell::Cell, path::Path};

use lexer::token::{Token, TokenKind};

use ariadne::{Label, Report, ReportKind, Source};
use thiserror::Error;

#[derive(Debug, Error)]
enum ParserError {
    #[error("Failed to parse Lambda program.")]
    ParseFail,
}

pub struct ParserErrorReporter;

// Specific error report handlers
impl ParserErrorReporter {
    pub fn unexpected_token<'a>(
        unexpected: &TokenKind,
        expected_toks: &[TokenKind],
        path: &str,
        source: &str,
        offset: usize,
    ) {
        let note = format!(
            "`{0:?}` is an unexpected token. Expected: `{1:?}`",
            unexpected, expected_toks
        );
        Report::build(ReportKind::Error, path, offset)
            .with_code(0)
            .with_message("Unexpected Token (syntax error)")
            .with_label(
                Label::new((path, offset..offset))
                    .with_message("Here")
                    .with_color(ariadne::Color::Red),
            )
            .with_note(note)
            .finish()
            .print((path, Source::from(source)))
            .unwrap();
    }
}

mod ast {
    use derive_new::new;
    use lexer::token::Token;

    #[derive(Debug, new)]
    pub struct Program {
        declarations: Option<Vec<Declaration>>,
        main: Main,
    }

    #[derive(Debug, new)]
    pub struct Main {
        signature: MainSignature,
        definition: MainDefinition,
    }

    #[derive(Debug, new)]
    pub struct MainSignature {
        name: Token,
        ty_list: Option<Vec<Token>>,
        ty_ret: Option<Token>,
    }

    #[derive(Debug, new)]
    pub struct MainDefinition {
        name: Token,
        arg_list: Option<Vec<Token>>,
        block: Block,
    }

    #[derive(Debug, new)]
    pub enum Declaration {
        Function {
            signature: FuncSignature,
            definition: FuncDefinition,
        },
        // TODO: Add support for enums and structs
        // Choice,
        // Struct
    }

    #[derive(Debug, new)]
    pub struct FuncSignature {
        func_name: Token,
        ty_list: Option<Vec<Token>>,
        ty_ret: Option<Token>,
    }

    #[derive(Debug, new)]
    pub struct FuncDefinition {
        func_name: Token,
        arg_list: Option<Vec<Token>>,
        block: Block,
    }

    #[derive(Debug, new)]
    pub struct Block {
        statements: Option<Vec<Statement>>,
        expression: Option<Expression>,
    }

    #[derive(Debug, new)]
    pub enum Statement {
        VarBinding { bind_name: Token, expr: Expression },
    }

    #[derive(Debug, new)]
    pub struct Expression {
        term: Term,
        other: Option<Vec<(TermOp, Term)>>,
    }

    #[derive(Debug, new)]
    pub struct Term {
        factor: Factor,
        other: Option<Vec<(FactOp, Factor)>>,
    }

    #[derive(Debug, new)]
    pub struct Factor {
        atom: Atom,
    }

    #[derive(Debug, new)]
    pub enum Atom {
        Token,

        // `(` Expr `)`
        Expression,
    }

    #[derive(Debug, new)]
    // `+` | `-`
    pub struct TermOp(Token);

    #[derive(Debug, new)]
    // `*` | `/`
    pub struct FactOp(Token);
}

pub struct Parser<'parser> {
    tokens: Vec<Token>,

    /// `pos` is a `Cell` to help limit mutation and to keep methods of `Parser` only needing
    /// `&self` instead of `&mut self`. This is a controlled form of mutation!
    pos: Cell<usize>,

    path: &'parser Path,

    cleaned_source: &'parser str,
}

/// CTOR for the `Parser`
impl<'parser> Parser<'parser> {
    pub fn new(tokens: Vec<Token>, path: &'parser Path, cleaned_source: &'parser str) -> Self {
        Self {
            tokens,
            pos: Cell::new(0),
            path,
            cleaned_source,
        }
    }
}

/// Internal helper functions to build smaller parsers  
impl Parser<'_> {
    /// Get the next upcoming `Token` by ownership based on current `Parser`
    /// index `pos`.
    fn peek(&self) -> Token {
        let curr_tok_pos = &self.pos;

        self.tokens[curr_tok_pos.get()].clone()
    }

    // `is_next_token()` sees if next token is what we assert it to be AND if we
    // are not at EOF.
    fn is_a(&self, kind: TokenKind) -> bool {
        let at_eof = self.pos.get() >= self.tokens.len();

        self.peek().get_token_kind() == kind && !at_eof
    }

    /// `advance_parser_pos()` moves the parser's position index by one.
    fn advance_parser_pos(&self) {
        // Make sure we do not increment passed the size of a `usize`
        let new_pos = self.pos.get() + 1;
        self.pos.set(new_pos);
    }
}

impl Parser<'_> {
    pub fn parse(&self) -> ast::Program {
        dbg!(&self.tokens);

        let declarations = self.parse_declarations();
        let main = self.parse_main();

        ast::Program::new(declarations, main)
    }

    fn parse_declarations(&self) -> Option<Vec<ast::Declaration>> {
        //ast::Declaration::new_function()

        // TODO: while-loop might be needed to create the whole Vector
        let curr_token = self.peek();
        let declaration = match curr_token
        {
            token if token.is_a(TokenKind::Ident) => self.parse_function_declaration(),
            // TODO: Add support for `struct` and `choice` decls
            _ => unreachable!(),
        };

        todo!()
    }

    fn parse_function_declaration(&self) -> Result<ast::Declaration, ParserError> {
        let function_signature = self.parse_function_signature();
        let function_definition = self.parse_function_definition();

        Ok(ast::Declaration::new_function(
            function_signature?,
            function_definition,
        ))
    }

    fn parse_function_signature(&self) -> Result<ast::FuncSignature, ParserError> {
        // Get name of function (e.g identifier)
        let func_name = self.try_consume(&[TokenKind::Ident])?;

        // See if TQualifier is given (e.g `::`) and ignore
        let _func_t_qualifier = self.try_consume(&[TokenKind::TQualifer])?;

        // See if left parenthesis is given (e.g `(`) and ignore
        let _l_parn = self.try_consume(&[TokenKind::LParn])?;

        // TODO: Add support for ADTs in function input types
        let types = &[
            // TokenKind::Adt,
            TokenKind::IntTy,
            TokenKind::BoolTy,
            TokenKind::TextTy,
            TokenKind::FloatTy,
        ];

        // Get function input types (e.g `int` or `bool`)
        let func_input_tys = self.try_consume_list(types)?;

        // See if right parenthesis is given (e.g `)`) and ignore
        let _r_parn = self.try_consume(&[TokenKind::RParn])?;

        // See if return arrow operator is given (e.g `->`) and ignore
        let _r_arrow = self.try_consume(&[TokenKind::RetArrow])?;

        // Get function return type
        let ret_ty = self.try_consume(types)?;

        Ok(ast::FuncSignature::new(
            func_name,
            Some(func_input_tys),
            Some(ret_ty),
        ))
    }

    fn parse_function_definition(&self) -> ast::FuncDefinition {
        unimplemented!();
    }

    fn parse_main(&self) -> ast::Main {
        todo!()
    }

    fn try_consume(&self, expected_token: &[TokenKind]) -> Result<Token, ParserError> {
        // Fetch next token in stream
        let curr_tok = self.peek();

        if !expected_token.contains(&curr_tok.get_token_kind())
        {
            // Print fancy compiler error
            ParserErrorReporter::unexpected_token(
                &curr_tok.get_token_kind(),
                expected_token.into(),
                self.path.to_str().unwrap(),
                self.cleaned_source,
                curr_tok.get_file_index(),
            );

            return Err(ParserError::ParseFail);
        }

        self.advance_parser_pos();
        Ok(curr_tok)
    }

    fn try_consume_list(&self, expected_token: &[TokenKind]) -> Result<Vec<Token>, ParserError> {
        // Fetch next token in stream
        let mut curr_tok = self.peek();

        // See if the next token even corresponds to what we expect
        if !expected_token.contains(&curr_tok.get_token_kind())
        {
            // Print fancy compiler error
            ParserErrorReporter::unexpected_token(
                &curr_tok.get_token_kind(),
                expected_token.into(),
                self.path.to_str().unwrap(),
                self.cleaned_source,
                curr_tok.get_file_index(),
            );

            return Err(ParserError::ParseFail);
        }

        // Accumulate tokens that we consume based on `expected_token`s
        let mut consumed_toks = Vec::new();
        while expected_token.contains(&curr_tok.get_token_kind()) || curr_tok.is_a(TokenKind::Sep)
        {
            // Skip separators
            if curr_tok.is_a(TokenKind::Sep)
            {
                self.advance_parser_pos();
                curr_tok = self.peek();
                continue;
            }

            consumed_toks.push(curr_tok);

            // Move parser index to next token
            self.advance_parser_pos();

            // Fetch next token
            curr_tok = self.peek();
        }

        Ok(consumed_toks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
