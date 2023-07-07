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
        let function_signature = self.parse_function_signature()?;
        let function_definition = self.parse_function_definition()?;

        Ok(ast::Declaration::new_function(
            function_signature,
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
        let func_input_tys = self.optional_consume_list(types);

        // See if right parenthesis is given (e.g `)`) and ignore
        let _r_parn = self.try_consume(&[TokenKind::RParn])?;

        // Consume either both return arrow and type, or neither (they can be omitted together)
        let (_r_arrow, ret_ty) = self.try_consume2_or_none(&[TokenKind::RetArrow], types)?;

        Ok(ast::FuncSignature::new(func_name, func_input_tys, ret_ty))
    }

    fn parse_function_definition(&self) -> Result<ast::FuncDefinition, ParserError> {
        // Get name of function (e.g identifier)
        let func_name = self.try_consume(&[TokenKind::Ident])?;

        // Get function parameters (e.g identifiers). Can omit entirely.
        let func_params = self.optional_consume_list(&[TokenKind::Ident]);

        // Check for and ignore function def operator (e.g `=`)
        let _func_def_op = self.try_consume(&[TokenKind::FnDef])?;

        // Check for and ignore opening of block (e.g `{`)
        let _block_open = self.try_consume(&[TokenKind::LBracket])?;

        // Parse out `Block`
        let block = self.parse_block()?;

        unimplemented!();
    }

    fn parse_block(&self) -> Result<ast::Block, ParserError> {
        let statements = self.parse_statements()?;
        let expression = self.parse_expression()?;
        unimplemented!();
    }

    fn parse_statements(&self) -> Result<Option<Vec<ast::Statement>>, ParserError> {
        let mut statements = Vec::new();

        // Loop to parse statements. Terminates 
        'parse_stmts: loop
        {
            // See what the statement starts with to determine what it is
            let curr_token = self.peek();
            let statement = match curr_token
            {
                token if token.is_a(TokenKind::LetKw) => self.parse_var_binding()?,
                // TODO: Add support for `struct` and `choice` decls
                _ if statements.is_empty() => return Ok(None),
                _ => break 'parse_stmts
            };

            statements.push(statement);
        }

        Ok(Some(statements))
    }

    fn parse_var_binding(&self) -> Result<ast::Statement, ParserError> {
        unimplemented!();
    }

    fn parse_main(&self) -> ast::Main {
        todo!()
    }

    // Tries to consume a single Token in stream with the provided set of Tokens that are
    // acceptable via `expected_token`
    fn try_consume(&self, valid_tokens: &[TokenKind]) -> Result<Token, ParserError> {
        // Fetch next token in stream
        let curr_tok = self.peek();

        if !valid_tokens.contains(&curr_tok.get_token_kind())
        {
            // Print fancy compiler error
            ParserErrorReporter::unexpected_token(
                &curr_tok.get_token_kind(),
                valid_tokens.into(),
                self.path.to_str().unwrap(),
                self.cleaned_source,
                curr_tok.get_file_index(),
            );

            return Err(ParserError::ParseFail);
        }

        self.advance_parser_pos();
        Ok(curr_tok)
    }

    // Tries to consume two Tokens based on `valid_tokens1` and `valid_tokens2` and returns either
    // given tokens `Ok((Some(token1), Some(token2)))` or omitted tokens `Ok((None, None))` or
    // fails with `Err(ParserError)`
    fn try_consume2_or_none(
        &self,
        valid_tokens1: &[TokenKind],
        valid_tokens2: &[TokenKind],
    ) -> Result<(Option<Token>, Option<Token>), ParserError> {
        let token1 = self.try_consume(valid_tokens1);
        let token2 = self.try_consume(valid_tokens2);

        // Tokens were optionally omitted, which is OK!
        if token1.is_err() && token2.is_err()
        {
            return Ok((None, None));
        }

        if token1.is_ok() && token2.is_err()
        {
            // Clone for the error to avoid move issues.
            // NOTE: note a performance issue since its an error case.
            let token2 = token2?.clone();

            // Print fancy compiler error
            ParserErrorReporter::unexpected_token(
                &token2.get_token_kind(),
                valid_tokens2.into(),
                self.path.to_str().unwrap(),
                self.cleaned_source,
                token2.get_file_index(),
            );

            return Err(ParserError::ParseFail);
        }

        if token1.is_err() && token2.is_ok()
        {
            // Clone for the error to avoid move issues.
            // NOTE: note a performance issue since its an error case.
            let token1 = token1?.clone();

            // Print fancy compiler error
            ParserErrorReporter::unexpected_token(
                &token1.get_token_kind(),
                valid_tokens1.into(),
                self.path.to_str().unwrap(),
                self.cleaned_source,
                token1.get_file_index(),
            );

            return Err(ParserError::ParseFail);
        }

        Ok((Some(token1?), Some(token2?)))
    }

    // Tries to optionally consume a single Token in stream with the provided set of Tokens that are
    // acceptable via `expected_token`. If it is not immediately found, just return `None`. No big
    // deal!
    fn optional_consume(&self, valid_tokens: &[TokenKind]) -> Option<Token> {
        // Fetch next token in stream
        let curr_tok = self.peek();

        if !valid_tokens.contains(&curr_tok.get_token_kind())
        {
            return None;
        }

        self.advance_parser_pos();
        Some(curr_tok)
    }

    fn try_consume_list(&self, valid_tokens: &[TokenKind]) -> Result<Vec<Token>, ParserError> {
        // Fetch next token in stream
        let mut curr_tok = self.peek();

        // See if the next token even corresponds to what we expect
        if !valid_tokens.contains(&curr_tok.get_token_kind())
        {
            // Print fancy compiler error
            ParserErrorReporter::unexpected_token(
                &curr_tok.get_token_kind(),
                valid_tokens.into(),
                self.path.to_str().unwrap(),
                self.cleaned_source,
                curr_tok.get_file_index(),
            );

            return Err(ParserError::ParseFail);
        }

        // Accumulate tokens that we consume based on `expected_token`s
        let mut consumed_toks = Vec::new();
        while valid_tokens.contains(&curr_tok.get_token_kind()) || curr_tok.is_a(TokenKind::Sep)
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

    fn optional_consume_list(&self, valid_tokens: &[TokenKind]) -> Option<Vec<Token>> {
        // Fetch next token in stream
        let mut curr_tok = self.peek();

        // See if the next token even corresponds to what we expect.
        // NOTE: it is OK to not find a match since lists can be empty!
        if !valid_tokens.contains(&curr_tok.get_token_kind())
        {
            return None;
        }

        // Accumulate tokens that we consume based on `expected_token`s
        let mut consumed_toks = Vec::new();
        while valid_tokens.contains(&curr_tok.get_token_kind()) || curr_tok.is_a(TokenKind::Sep)
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

        Some(consumed_toks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
