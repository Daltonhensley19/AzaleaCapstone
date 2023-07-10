//! AST parser for the Morehead Lambda Compiler
//!
//! NOTE: The formal grammar is defined in the `grammar/` directory inside the file
//! `formal_grammar.pest`.

use std::{cell::Cell, path::Path};

use lexer::token::{Token, TokenKind};

use ariadne::{Label, Report, ReportKind, Source};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
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

    pub fn var_bind_missing_rhs<'a>(
        var_bind_name: &Token,
        path: &str,
        source: &str,
        offset: usize,
    ) {
        let note = format!(
            "`{0:?}` is missing expression after `<-`.",
            var_bind_name.get_raw_content()
        );
        Report::build(ReportKind::Error, path, offset)
            .with_code(0)
            .with_message("Let Binding Incomplete (syntax error)")
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

    pub fn incomplete_binary_op<'a>(path: &str, source: &str, offset: usize) {
        let note = format!("`Binary Operation is incomplete (syntax error)");
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
        term: Box<Term>,
        other: Option<Vec<(TermOp, Term)>>,
    }

    #[derive(Debug, new)]
    pub struct Term {
        factor: Option<Factor>,
        other: Option<Vec<(FactOp, Factor)>>,
    }

    impl Term {
        pub fn is_none(&self) -> bool {
            self.factor.is_none()
        }
    }

    #[derive(Debug, new)]
    pub struct Factor {
        atom: Option<Atom>,
    }

    #[derive(Debug, new)]
    pub enum Atom {
        Token(Token),

        // `(` Expr `)`
        Expression(Option<Expression>),
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
    fn peek(&self) -> Option<Token> {
        let curr_tok_pos = &self.pos;

        let tok = self.tokens.get(curr_tok_pos.get()).clone();

        tok.cloned()
    }

    // `is_next_token()` sees if next token is what we assert it to be AND if we
    // are not at EOF.
    fn is_a(&self, kind: TokenKind) -> bool {
        let at_eof = self.pos.get() >= self.tokens.len();

        self.peek().unwrap().get_token_kind() == kind && !at_eof
    }

    /// `advance_parser_pos()` moves the parser's position index by one.
    fn advance_parser_pos(&self) {
        // Make sure we do not increment passed the size of a `usize`
        let new_pos = self.pos.get() + 1;

        self.pos.set(new_pos);
    }

    fn at_end_of_token_stream(&self) -> bool {
        let stream_size = self.tokens.len();

        self.pos.get() >= stream_size
    }
}

impl Parser<'_> {
    pub fn parse(&self) -> Result<ast::Program, ParserError> {
        let declarations = self.parse_declarations()?;

        dbg!(&declarations);

        let main = self.parse_main();

        Ok(ast::Program::new(declarations, main))
    }

    fn parse_declarations(&self) -> Result<Option<Vec<ast::Declaration>>, ParserError> {
        use TokenKind::*;
        let mut declarations = Vec::new();

        // Loop to parse declarationa. Terminates
        'parse_decls: loop
        {
            // Halt parsing if we reach the end of the file
            if self.at_end_of_token_stream()
            {
                break 'parse_decls;
            }

            // See what the declaration starts with to determine what it is
            let curr_token = self.try_peek(&[Ident, StructKw, ChoiceKw]);

            if curr_token.is_err() && declarations.is_empty()
            {
                return Ok(None);
            }

            let curr_token = curr_token.unwrap();

            let declaration = match curr_token.get_token_kind()
            {
                Ident => self.parse_function_declaration()?,
                // @todo: Add support for `struct` and `choice` decls
                _ => break 'parse_decls,
            };

            declarations.push(declaration);
        }

        Ok(Some(declarations))
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
        use TokenKind::*;

        // Get name of function (e.g identifier)
        let func_name = self.try_consume(&[Ident])?;

        // See if TQualifier is given (e.g `::`) and ignore
        let _func_t_qualifier = self.try_consume(&[TQualifer])?;

        // See if left parenthesis is given (e.g `(`) and ignore
        let _l_parn = self.try_consume(&[LParn])?;

        // @todo: Add support for ADTs in function input types
        let types = &[
            // TokenKind::Adt,
            IntTy, BoolTy, TextTy, FloatTy,
        ];

        // Get function input types (e.g `int` or `bool`)
        let func_input_tys = self.optional_consume_list(types);

        // See if right parenthesis is given (e.g `)`) and ignore
        let _r_parn = self.try_consume(&[RParn])?;

        // Consume either both return arrow and type, or neither (they can be omitted together)
        let (_r_arrow, ret_ty) = self.try_consume2_or_none(&[RetArrow], types)?;

        Ok(ast::FuncSignature::new(func_name, func_input_tys, ret_ty))
    }

    fn parse_function_definition(&self) -> Result<ast::FuncDefinition, ParserError> {
        use TokenKind::*;

        // Get name of function (e.g identifier)
        let func_name = self.try_consume(&[Ident])?;

        // Get function parameters (e.g identifiers). Can omit entirely.
        let func_params = self.optional_consume_list(&[Ident]);

        // Check for and ignore function def operator (e.g `=`)
        let _func_def_op = self.try_consume(&[FnDef])?;

        // Check for and ignore opening of block (e.g `{`)
        let _block_open = self.try_consume(&[LBracket])?;

        // Parse out `Block`
        let block = self.parse_block()?;

        let _block_close = self.try_consume(&[RBracket])?;

        Ok(ast::FuncDefinition::new(func_name, func_params, block))
    }

    fn parse_block(&self) -> Result<ast::Block, ParserError> {
        let statements = self.parse_statements()?;
        let expression = self.parse_expression()?;

        Ok(ast::Block::new(statements, expression))
    }

    fn parse_statements(&self) -> Result<Option<Vec<ast::Statement>>, ParserError> {
        use TokenKind::*;

        let mut statements = Vec::new();

        // Loop to parse statements. Terminates
        'parse_stmts: loop
        {
            // See what the statement starts with to determine what it is
            let curr_token = self.optional_peek(&[LetKw, StructKw, ChoiceKw]);

            if curr_token.is_none() && statements.is_empty()
            {
                return Ok(None);
            }

            if curr_token.is_none() && !statements.is_empty()
            {
                return Ok(Some(statements));
            }

            let curr_token = curr_token.unwrap();

            let statement = match curr_token.get_token_kind()
            {
                LetKw => self.parse_var_binding()?,
                // @todo: Add support for `struct` and `choice` decls
                _ if statements.is_empty() => return Ok(None),
                _ => break 'parse_stmts,
            };

            statements.push(statement);
        }

        Ok(Some(statements))
    }

    fn parse_var_binding(&self) -> Result<ast::Statement, ParserError> {
        use TokenKind::*;

        let _let_kw = self.try_consume(&[LetKw])?;

        let var_bind_name = self.try_consume(&[Ident])?;

        let _assign_op = self.try_consume(&[Assign])?;

        let rhs = self.parse_expression()?;

        if rhs.is_none()
        {
            // Fancy compiler error
            // Print fancy compiler error
            ParserErrorReporter::var_bind_missing_rhs(
                &var_bind_name,
                self.path.to_str().unwrap(),
                self.cleaned_source,
                var_bind_name.get_file_index(),
            );

            return Err(ParserError::ParseFail);
        };

        let _semicolon = self.try_consume(&[Semicolon]);

        Ok(ast::Statement::new_var_binding(var_bind_name, rhs.unwrap()))
    }

    fn parse_expression(&self) -> Result<Option<ast::Expression>, ParserError> {
        let term = self.parse_term()?;
        let other = self.parse_other_term()?;

        if term.is_none()
        {
            assert!(other.is_none());
            return Ok(None);
        }

        Ok(Some(ast::Expression::new(Box::new(term), other)))
    }

    fn parse_term(&self) -> Result<ast::Term, ParserError> {
        let factor = self.parse_factor()?;
        let other = self.parse_other_factor()?;

        Ok(ast::Term::new(factor, other))
    }

    fn parse_other_term(&self) -> Result<Option<Vec<(ast::TermOp, ast::Term)>>, ParserError> {
        use TokenKind::*;

        let mut other = Vec::new();

        'other_term: loop
        {
            let term_op = self.optional_consume(&[Plus, Minus]);
            if term_op.is_some()
            {
                let term = self.parse_term()?;

                // Report error if binary operator is missing its RHS
                if term.is_none()
                {
                    // Print fancy compiler error
                    ParserErrorReporter::incomplete_binary_op(
                        self.path.to_str().unwrap(),
                        self.cleaned_source,
                        term_op.unwrap().get_file_index(),
                    );

                    return Err(ParserError::ParseFail);
                }

                let term_op = ast::TermOp::new(term_op.unwrap());

                other.push((term_op, term))
            }
            else
            {
                break 'other_term;
            }
        }

        if other.is_empty()
        {
            Ok(None)
        }
        else
        {
            Ok(Some(other))
        }
    }

    fn parse_factor(&self) -> Result<Option<ast::Factor>, ParserError> {
        let atom = self.parse_atom()?;

        if atom.is_none()
        {
            return Ok(None);
        }

        Ok(Some(ast::Factor::new(atom)))
    }

    fn parse_other_factor(&self) -> Result<Option<Vec<(ast::FactOp, ast::Factor)>>, ParserError> {
        use TokenKind::*;

        let mut other = Vec::new();

        'other_factor: loop
        {
            let fact_op = self.optional_consume(&[Mul, Div]);
            if fact_op.is_some()
            {
                let Some(factor) = self.parse_factor()?
                else
                {
                    // Print fancy compiler error
                    ParserErrorReporter::incomplete_binary_op(
                        self.path.to_str().unwrap(),
                        self.cleaned_source,
                        fact_op.unwrap().get_file_index(),
                    );

                    return Err(ParserError::ParseFail);
                };

                let fact_op = ast::FactOp::new(fact_op.unwrap());

                other.push((fact_op, factor))
            }
            else
            {
                break 'other_factor;
            }
        }

        if other.is_empty()
        {
            Ok(None)
        }
        else
        {
            Ok(Some(other))
        }
    }

    fn parse_atom(&self) -> Result<Option<ast::Atom>, ParserError> {
        use TokenKind::*;

        // Get ident or literal for atom
        let curr_tok = self.optional_consume(&[Ident, NumLit, FloatLit, BoolLit, LParn]);

        if curr_tok.is_none()
        {
            return Ok(None);
        }

        match curr_tok.clone().unwrap().get_token_kind()
        {
            // @todo: Look to see if we need to call `advance_parser_pos()`
            LParn =>
            {
                // Parse `(` `inner_expr` `)`
                let inner_expr = Ok(Some(ast::Atom::new_expression(self.parse_expression()?)));

                // Make sure that user remembers to close with `)`
                let _ = self.try_consume(&[RParn])?;

                inner_expr
            }
            Ident | NumLit | FloatLit | BoolLit =>
            {
                Ok(Some(ast::Atom::new_token(curr_tok.unwrap())))
            }
            _ => Ok(None),
        }
    }

    fn parse_main(&self) -> ast::Main {
        todo!()
    }

    // Tries to consume a single Token in stream with the provided set of Tokens that are
    // acceptable via `expected_token`
    fn try_consume(&self, valid_tokens: &[TokenKind]) -> Result<Token, ParserError> {
        // Fetch next token in stream
        let curr_tok = self.peek().unwrap();

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

    // Tries to peek a single Token in stream with the provided set of Tokens that are
    // acceptable via `expected_token` without advancing pos in token stream
    fn try_peek(&self, valid_tokens: &[TokenKind]) -> Result<Token, ParserError> {
        // Fetch next token in stream
        let curr_tok = self.peek().unwrap();

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
        let curr_tok = self.peek().unwrap();

        if !valid_tokens.contains(&curr_tok.get_token_kind())
        {
            return None;
        }

        self.advance_parser_pos();
        Some(curr_tok)
    }

    // Tries to optionally peek a single Token in stream with the provided set of Tokens that are
    // acceptable via `expected_token`. If it is not immediately found, just return `None`. No big
    // deal! Note: same as `optional_consume()`, however we do not advance token pos index
    fn optional_peek(&self, valid_tokens: &[TokenKind]) -> Option<Token> {
        // Fetch next token in stream
        let curr_tok = self.peek().unwrap();

        if !valid_tokens.contains(&curr_tok.get_token_kind())
        {
            return None;
        }

        Some(curr_tok)
    }

    fn try_consume_list(&self, valid_tokens: &[TokenKind]) -> Result<Vec<Token>, ParserError> {
        // Fetch next token in stream
        let mut curr_tok = self.peek().unwrap();

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
                curr_tok = self.peek().unwrap();
                continue;
            }

            consumed_toks.push(curr_tok);

            // Move parser index to next token
            self.advance_parser_pos();

            // Fetch next token
            curr_tok = self.peek().unwrap();
        }

        Ok(consumed_toks)
    }

    fn optional_consume_list(&self, valid_tokens: &[TokenKind]) -> Option<Vec<Token>> {
        // Fetch next token in stream
        let mut curr_tok = self.peek().unwrap();

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
                curr_tok = self.peek().unwrap();
                continue;
            }

            consumed_toks.push(curr_tok);

            // Move parser index to next token
            self.advance_parser_pos();

            // Fetch next token
            curr_tok = self.peek().unwrap();
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
