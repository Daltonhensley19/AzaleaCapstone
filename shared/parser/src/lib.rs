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

    pub fn missing_ret_ty<'a>(
        unexpected: &TokenKind,
        expected_toks: &[TokenKind],
        path: &str,
        source: &str,
        offset: usize,
    ) {
        let note = format!(
            "`{0:?}` expected `{1:?}`, but no return type was given",
            unexpected, expected_toks
        );
        Report::build(ReportKind::Error, path, offset)
            .with_code(0)
            .with_message("Missing Return Type (syntax error)")
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

    pub fn missing_sep<'a>(unexpected: &TokenKind, path: &str, source: &str, offset: usize) {
        let note = format!(
            "`{0:?}` was unexpected. Expected to see a comma `,`",
            unexpected
        );
        Report::build(ReportKind::Error, path, offset)
            .with_code(0)
            .with_message("Missing Comma In List (syntax error)")
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
            .with_message("Binding Incomplete (syntax error)")
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
    }
    
    #[derive(Debug, new)]
    pub enum Declaration {
        Function {
            signature: FuncSignature,
            definition: FuncDefinition,
        },

        // TODO: Add support for enums and structs
        Choice {
            name: Token,
            variants: Option<Vec<Token>>,
        },

        Struct {
            name: Token,
            // Tuple is `(field_name, field_type)`
            typed_fields: Option<Vec<(Token, Token)>>,
        },
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
        VarBindingInit { bind_name: Token, expr: Expression },
        VarBindingMut { bind_name: Token, expr: Expression },
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
    /// Get the current `Token` by ownership based on current `Parser`
    /// index `pos`.
    fn peek(&self) -> Option<Token> {
        let curr_tok_pos = &self.pos;

        let tok = self.tokens.get(curr_tok_pos.get()).clone();

        tok.cloned()
    }

    /// Get the next `Token` by ownership based on current `Parser`
    /// index `pos`.
    fn peek_next(&self) -> Option<Token> {
        let curr_tok_pos = &self.pos;

        let tok = self.tokens.get(curr_tok_pos.get().wrapping_add(1)).clone();

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
        let new_pos = self.pos.get().saturating_add(1);

        self.pos.set(new_pos);
    }

    fn increment_parser_pos_by(&self, incre: usize) {
        // Make sure we do not increment passed the size of a `usize`
        let new_pos = self.pos.get().saturating_add(incre);

        self.pos.set(new_pos);
    }

   fn decrement_parser_pos_by(&self, decre: usize) {
        // Make sure we do not decrment passed the size of a `usize`
        let new_pos = self.pos.get().saturating_sub(decre);

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

        Ok(ast::Program::new(declarations))
    }

    fn parse_declarations(&self) -> Result<Option<Vec<ast::Declaration>>, ParserError> {
        use TokenKind::*;
        let mut declarations = Vec::new();

        // Loop to parse declarationa. Terminates
        'parse_decls: loop
        {
            // Try to get the name of the declaration
            let curr_token = self.try_consume(&[Ident, MainKw, EOF]);

            if curr_token.is_err() && declarations.is_empty()
            {
                return Ok(None);
            }

            let name_token = curr_token.unwrap();

            // Escape if we reach EOF token
            if name_token.is_a(EOF)
            {
                break 'parse_decls; 
            }

            let declaration = match name_token.get_token_kind()
            {
                Ident | MainKw =>  {
                    let _decl_t_qualifier = self.try_consume(&[TQualifer])?;

                    // `LParn` if the start of a function declaration
                    let decl_tok = self.try_consume(&[StructKw, ChoiceKw, LParn])?;

                    if decl_tok.is_a(StructKw) 
                    {
                        // Reposition stream position 
                        self.decrement_parser_pos_by(3);

                        self.parse_struct_declaration()?
                    }
                    else if decl_tok.is_a(ChoiceKw)
                    {
                         // Reposition stream position 
                        self.decrement_parser_pos_by(3);

                        self.parse_choice_declaration()?
                    }
                    else 
                    {
                        // Reposition stream position 
                        self.decrement_parser_pos_by(3);

                        self.parse_function_declaration()?
                    }
                },
                
                // @todo: Add support for `struct` and `choice` decls
                _ => break 'parse_decls,
            };

            declarations.push(declaration);
        }

        Ok(Some(declarations))
    }

    fn parse_choice_declaration(&self) -> Result<ast::Declaration, ParserError> {
        use TokenKind::*;

        // Get choice name 
        let choice_name = self.try_consume(&[Ident])?;

        // Check for TQualifier
        let _choice_t_qualifier = self.try_consume(&[TQualifer])?;

        // Check for choicekw 
        let _choice_kw = self.try_consume(&[ChoiceKw])?;

        // Check for opening left bracket for choice
        let _choice_l_bracket = self.try_consume(&[LBracket])?;

        // Get choice variants
        let choice_variants = self.try_optional_consume_list_with_seps(&[Ident])?;

        // Check for closing right bracket for choice
        let _choice_r_bracket = self.try_consume(&[RBracket])?;

        Ok(ast::Declaration::new_choice(choice_name, choice_variants))
    }

    fn parse_struct_declaration(&self) -> Result<ast::Declaration, ParserError> {
        use TokenKind::*;

        // Get structure name 
        let struct_name = self.try_consume(&[Ident])?;

        // Check for TQualifier
        let _struct_t_qualifier = self.try_consume(&[TQualifer])?;

        // Check for structkw 
        let _struct_kw = self.try_consume(&[StructKw])?;
 
        // Check for opening left bracket for structure (i.e. `{`)
        let _struct_l_bracket = self.try_consume(&[LBracket])?;

        // Get structure fields (i.e. `age :: int, name :: text`)
        let struct_fields = self.try_optional_consume_typed_list_with_seps()?;

        // Check for closing right bracket for structure (i.e. `}`)
        let _struct_r_bracket = self.try_consume(&[RBracket])?;

        Ok(ast::Declaration::new_struct(struct_name, struct_fields))
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
        let func_name = self.try_consume(&[Ident, MainKw])?;

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
        let func_name = self.try_consume(&[Ident, MainKw])?;

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
            // See what the statement starts with to determine what it is.
            // `Ident` is allowed since we could be parsing `VarBindingMut`.
            let curr_token = self.optional_peek(&[LetKw, StructKw, ChoiceKw, Ident]);

            // No statements to parse
            if curr_token.is_none() && statements.is_empty()
            {
                return Ok(None);
            }

            // *No more* statements to parse
            if curr_token.is_none() && !statements.is_empty()
            {
                return Ok(Some(statements));
            }

            let curr_token = curr_token.unwrap();

            // Parse statement
            let statement = match curr_token.get_token_kind()
            {
                LetKw => self.parse_var_binding_init()?,
                // Parse `VarBindingMut` if current is `Ident` and next is `<-`
                Ident if self.optional_peek_next(&[Assign]).is_some() =>
                {
                    self.parse_var_binding_mutation()?
                }
                // @todo: Add support for `struct` and `choice` decls
                _ if statements.is_empty() => return Ok(None),
                _ => break 'parse_stmts,
            };

            // Store in AST
            statements.push(statement);
        }

        Ok(Some(statements))
    }

    fn parse_var_binding_init(&self) -> Result<ast::Statement, ParserError> {
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

        let _semicolon = self.try_consume(&[Semicolon])?;

        Ok(ast::Statement::new_var_binding_init(
            var_bind_name,
            rhs.unwrap(),
        ))
    }

    fn parse_var_binding_mutation(&self) -> Result<ast::Statement, ParserError> {
        use TokenKind::*;

        //let _let_kw = self.try_consume(&[LetKw])?;

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

        let _semicolon = self.try_consume(&[Semicolon])?;

        Ok(ast::Statement::new_var_binding_mut(
            var_bind_name,
            rhs.unwrap(),
        ))
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
    
    fn try_consume_ty(&self) -> Result<Token, ParserError> {
        use TokenKind::*;

        // Fetch next token
        // @todo: add support for ADTs
        let ty_token = self.try_consume(&[IntTy, FloatTy, BoolTy, TextTy])?;

        Ok(ty_token)
    }

    fn try_optional_consume_typed_ident(&self) -> Result<Option<(Token, Token)>, ParserError> {
        use TokenKind::*;

        // Fetch next token in stream
        let curr_tok = self.optional_peek(&[Ident]);

        if let Some(ident_tok) = curr_tok
        {
            // Advance to next token
            self.advance_parser_pos();

            // Make sure user writes the `::` for explict type
            let _t_qualifier_tok = self.try_consume(&[TQualifer])?;

            // Get explict type
            let explicit_ty = self.try_consume_ty()?;

            Ok(Some((ident_tok, explicit_ty)))
        }
        else
        {
            Ok(None)
        }
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
        let token1 = self.optional_consume(valid_tokens1);
        let token2 = self.optional_consume(valid_tokens2);

        // Tokens were optionally omitted, which is OK!
        if token1.is_none() && token2.is_none()
        {
            return Ok((None, None));
        }

        if token1.is_some() && token2.is_none()
        {
            // Clone for the error to avoid move issues.
            // NOTE: note a performance issue since its an error case.
            let token1 = token1.unwrap().clone();

            // Print fancy compiler error
            ParserErrorReporter::missing_ret_ty(
                &token1.get_token_kind(),
                valid_tokens2.into(),
                self.path.to_str().unwrap(),
                self.cleaned_source,
                token1.get_file_index(),
            );

            return Err(ParserError::ParseFail);
        }

        if token1.is_none() && token2.is_some()
        {
            // Clone for the error to avoid move issues.
            // NOTE: note a performance issue since its an error case.
            let token2 = token2.unwrap().clone();

            // Print fancy compiler error
            ParserErrorReporter::unexpected_token(
                &token2.get_token_kind(),
                valid_tokens1.into(),
                self.path.to_str().unwrap(),
                self.cleaned_source,
                token2.get_file_index(),
            );

            return Err(ParserError::ParseFail);
        }

        Ok((Some(token1.unwrap()), Some(token2.unwrap())))
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

    // Tries to optionally peek *the next* single Token in stream with the provided set of Tokens that are
    // acceptable via `expected_token`. If it is not immediately found, just return `None`. No big
    // deal! Note: same as `optional_consume()`, however we do not advance token pos index
    fn optional_peek_next(&self, valid_tokens: &[TokenKind]) -> Option<Token> {
        // Fetch next token in stream
        let curr_tok = self.peek_next().unwrap();

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

    fn try_optional_consume_list_with_seps(
        &self,
        valid_tokens: &[TokenKind],
    ) -> Result<Option<Vec<Token>>, ParserError> {
        // Fetch next token in stream
        let mut curr_tok = self.peek().unwrap();

        // See if the next token even corresponds to what we expect.
        // NOTE: it is OK to not find a match since lists can be empty!
        if !valid_tokens.contains(&curr_tok.get_token_kind())
        {
            return Ok(None);
        }

        // Accumulate tokens that we consume based on `expected_token`s
        let mut consumed_toks = Vec::new();
        let mut expected_sep = false;
        while valid_tokens.contains(&curr_tok.get_token_kind()) || curr_tok.is_a(TokenKind::Sep)
        {
            // Skip separators if we expect them
            if curr_tok.is_a(TokenKind::Sep) && expected_sep
            {
                self.advance_parser_pos();
                curr_tok = self.peek().unwrap();

                // Next token should not be a separator
                expected_sep = false;
                continue;
            }

            // error detected -- Missing comma in list
            if !curr_tok.is_a(TokenKind::Sep) && expected_sep
            {
                // Fancy compiler error
                ParserErrorReporter::missing_sep(
                    &curr_tok.get_token_kind(),
                    self.path.to_str().unwrap(),
                    self.cleaned_source,
                    curr_tok.get_file_index(),
                );
            }

            // error detected -- Erroneous comma found in list
            if curr_tok.is_a(TokenKind::Sep) && !expected_sep
            {
                // Fancy compiler error
                ParserErrorReporter::unexpected_token(
                    &curr_tok.get_token_kind(),
                    valid_tokens.into(),
                    self.path.to_str().unwrap(),
                    self.cleaned_source,
                    curr_tok.get_file_index(),
                );
            }

            consumed_toks.push(curr_tok);

            // Move parser index to next token
            self.advance_parser_pos();

            // Fetch next token
            curr_tok = self.peek().unwrap();

            // Next token should be a comma seperator
            expected_sep = true;
        }

        Ok(Some(consumed_toks))
    }

    fn try_optional_consume_typed_list_with_seps(
        &self,
    ) -> Result<Option<Vec<(Token, Token)>>, ParserError> {
        use TokenKind::*;

        // Accumulate tokens that we consume based on `expected_token`s
        let mut consumed_ty_idents = Vec::new();
        'build_ty_idents: loop
        {
            // Try to fetch typed ident in stream
            let Some((ident_tok, ty_tok)) = self.try_optional_consume_typed_ident()?
            else
            {
                return Ok(None);
            };

            // Make sure we have a seperator
            let curr_tok = self.optional_consume(&[Sep]);

            // Store typed ident and go to top of loop to get more typed idents
            if curr_tok.is_some()
            {
                consumed_ty_idents.push((ident_tok, ty_tok));

                continue 'build_ty_idents;
            }

            // Move parser index to next token
            //self.advance_parser_pos();

            // If there is not a comma seperator, stop parsing typed idents
            if curr_tok.is_none()
            {
                break 'build_ty_idents;
            }
        }

        Ok(Some(consumed_ty_idents))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
