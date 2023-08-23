use std::{cell::Cell, path::Path};

use crate::ast;
use crate::ast::Type;
use crate::errors::{ParserErrorReporter, ParserError};

use lexer::token::{Token, TokenKind};


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
    pub fn parse(&self, verbose: bool) -> Result<ast::Program, ParserError> {
        let declarations = self.parse_declarations()?;

        if verbose 
        {
            println!("[Generated AST]:");
            dbg!(&declarations);
        }

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
                Ident | MainKw =>
                {
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
                }

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

        // Parse out `Block`
        let block = self.parse_block()?;

        Ok(ast::FuncDefinition::new(func_name, func_params, block))
    }

    fn parse_block(&self) -> Result<ast::Block, ParserError> {
	use TokenKind::*;
	
	let _block_open       = self.try_consume(&[LBracket])?;
        let statements        = self.parse_statements()?;
	let min_binding_power = 0;
        let expression        = self.parse_expression(min_binding_power)?;
	let _block_close      = self.try_consume(&[RBracket])?;

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
            let curr_token = self.optional_peek(&[IfKw,
						  WhileKw,
						  ForKw,
						  LetKw,
						  StructKw,
						  ChoiceKw,
						  Ident]);

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
                LetKw   => self.parse_var_binding_init()?,
                IfKw    => self.parse_selection()?,
                WhileKw => self.parse_indefinite_loop()?,
                ForKw   => self.parse_definite_loop()?,
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

    fn parse_definite_loop(&self) -> Result<ast::Statement, ParserError> {
	use TokenKind::*;

	// Parse for-loop
        let _for_kw        = self.try_consume(&[ForKw])?;
        let for_index      = self.try_consume(&[Ident])?;
        let _in_kw         = self.try_consume(&[InKw])?;
        let for_low_bound  = self.try_consume(&[NumLit])?;
        let _for_range     = self.try_consume(&[ExRange])?;
        let for_high_bound = self.try_consume(&[NumLit])?;
	let for_block      = self.parse_block()?;

        Ok(ast::Statement::new_definite_loop(for_index, for_low_bound, for_high_bound, for_block))
    }
    
    fn parse_indefinite_loop(&self) -> Result<ast::Statement, ParserError> {
	use TokenKind::*;

	// Parse while-loop
        let while_kw          = self.try_consume(&[WhileKw])?;
	let min_binding_power = 0;
	let Some(while_expr)  = self.parse_expression(min_binding_power)?
	else
	{
	    // Fancy compiler error
            ParserErrorReporter::missing_expr_at(
		"while-loop",
                self.path.to_str().unwrap(),
                self.cleaned_source,
                while_kw.get_file_index(),
            );

	    return Err(ParserError::ParseFail);
	};
	let while_block  = self.parse_block()?;

        Ok(ast::Statement::new_indefinite_loop(while_expr, while_block))
    }

    fn parse_selection(&self) -> Result<ast::Statement, ParserError> {
        use TokenKind::*;

	// Parse `if-comp` 
	let if_comp = self.parse_if_comp()?;

	// Parse `elif-comp` 
        let elif_comp = self.parse_elif_comp()?;

	// Parse `else-comp` 
        let else_comp = self.parse_else_comp()?;

        Ok(ast::Statement::new_selection(if_comp, elif_comp, else_comp))
    }

    fn parse_if_comp(&self) -> Result<ast::IfComp, ParserError> {
	use TokenKind::*;

	// Parse `if-comp`
	let if_kw             = self.try_consume(&[IfKw])?;
	let min_binding_power = 0;
	let Some(if_expr)     = self.parse_expression(min_binding_power)?
	else
	{
	    // Fancy compiler error
            ParserErrorReporter::missing_expr_at(
		"if-branch",
                self.path.to_str().unwrap(),
                self.cleaned_source,
                if_kw.get_file_index(),
            );

	    return Err(ParserError::ParseFail);
	};
        let if_block     = self.parse_block()?;
        let if_comp      = ast::IfComp::new(if_expr, if_block);

        Ok(if_comp)
    }

    fn parse_elif_comp(&self) -> Result<Option<ast::ElifComp>, ParserError> {
        use TokenKind::*;
	
	let _elif_kw = self.optional_peek(&[ElifKw]);

	// No `elif` to parse
	if _elif_kw.is_none()
	{
	    return Ok(None);
	}

	// Parse `elif-comp` 
	let elif_kw           = self.try_consume(&[ElifKw])?;
	let min_binding_power = 0; 
	let Some(elif_expr)   = self.parse_expression(min_binding_power)?
	else
	{
	    // Fancy compiler error
            ParserErrorReporter::missing_expr_at(
		"elif-branch",
                self.path.to_str().unwrap(),
                self.cleaned_source,
                elif_kw.get_file_index(),
            );

	    return Err(ParserError::ParseFail);
	};
        let elif_block     = self.parse_block()?;
        let elif_comp      = Some(ast::ElifComp::new(elif_expr, elif_block));

	Ok(elif_comp)
    }

    fn parse_else_comp(&self) -> Result<Option<ast::ElseComp>, ParserError> {
        use TokenKind::*;
	
	let _else_kw = self.optional_peek(&[ElseKw]);

	// No `else` to parse
	if _else_kw.is_none()
	{
	    return Ok(None);
	}

	// Parse `else-comp` 
        let _else_kw       = self.try_consume(&[ElseKw])?;
        let else_block     = self.parse_block()?;
        let else_comp      = Some(ast::ElseComp::new(else_block));

	Ok(else_comp)
    }

    
    fn parse_var_binding_init(&self) -> Result<ast::Statement, ParserError> {
        use TokenKind::*;

        let _let_kw = self.try_consume(&[LetKw])?;

        let var_bind_name = self.try_consume(&[Ident])?;

	// Try to see if user supplied ty-hint and get it.
	// We either have `("::", ty_hint)` or `(None, None)` or error.
	let types = &[Ident, IntTy, FloatTy, TextTy, BoolTy];
	let (_t_qualifier, ty_hint) = self.try_consume2_or_none(&[TQualifer], types)?;

	// If hint was supplied, we have `Some(type_hint)`. Otherwise, `None`.
	let ty_hint = if ty_hint.is_some() { Some(Type(ty_hint.unwrap())) } else { None };
	
        let _assign_op = self.try_consume(&[Assign])?;

	let min_binding_power = 0;
        let rhs = self.parse_expression(min_binding_power)?;

	println!("S-Expr: `{rhs}`", rhs=rhs.clone().unwrap());

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
	    ty_hint,
            rhs.unwrap(),
        ))
    }

    fn parse_var_binding_mutation(&self) -> Result<ast::Statement, ParserError> {
        use TokenKind::*;

        //let _let_kw = self.try_consume(&[LetKw])?;

        let var_bind_name = self.try_consume(&[Ident])?;

        let _assign_op = self.try_consume(&[Assign])?;

	let min_binding_power = 0;
        let rhs = self.parse_expression(min_binding_power)?;

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
    
    fn get_prefix_bind_power(op: &Token) -> ((), u8) {
	use TokenKind::*;

	// Note: binding power gives us precedence AND associatvity 
	match op {
            tok if tok.is_a(Minus) => ((), 13),
            bad_tok => panic!("Unsupported op: {:?}", bad_tok)
	}
    }

    fn get_infix_bind_power(op: &Token) -> Option<(u8, u8)> {
	use TokenKind::*;

	// Note: binding power gives us precedence AND associatvity 
	let infix_bp = match op {
            tok if tok.is_a(OrKw)  => (1, 2),
            tok if tok.is_a(AndKw) => (3, 4),
            tok if tok.is_a(Eq)    => (5, 6),
            tok if tok.is_a(Lt)    => (5, 6),
            tok if tok.is_a(Lte)   => (5, 6),
            tok if tok.is_a(Gt)    => (5, 6),
            tok if tok.is_a(Gte)   => (5, 6),
            tok if tok.is_a(Plus)  => (7, 8),
            tok if tok.is_a(Minus) => (7, 8),
            tok if tok.is_a(Mul)   => (9, 10),
            tok if tok.is_a(Div)   => (9, 10),
	    tok if tok.is_a(AsKw)  => (11, 12),
            bad_tok => panic!("Unsupported op: {:?}", bad_tok)
	};

	Some(infix_bp)
    }

    fn get_postfix_bind_power(op: &Token) -> Option<(u8, ())> {
	use TokenKind::*;

	// Note: binding power gives us precedence AND associatvity 
	let postfix_bp = match op {
            tok if tok.is_a(LSBracket) => (15, ()),
            _ => return None,
	};

	Some(postfix_bp)
    }

    // Pratt parsing of expressions into S-Expressions
    fn parse_expression(&self, minimum_bp: u8) -> Result<Option<ast::Expression>, ParserError> {
	use TokenKind::*;

	// Parse LHS of expression 
	let op_kind    = &[Minus]; 
	let ty_kind    = &[Ident, FloatTy, IntTy, BoolTy, TextTy];
	let value_kind = &[BoolLit, NumLit, FloatLit];
	let punc_kind  = &[RBracket, Semicolon, LParn];
	let all_kind   = &[&op_kind[..], &ty_kind[..], &value_kind[..], &punc_kind[..]].concat();
	let mut lhs = match self.try_consume(&all_kind)? {
            good_tok if good_tok.is_a(RBracket)  => {self.decrement_parser_pos_by(1); return Ok(None);},
	    good_tok if good_tok.is_a(Semicolon) => {self.decrement_parser_pos_by(1); return Ok(None);},
	    // `(` Expression `)` support 
	    good_tok if good_tok.is_a(LParn) => {
		let min_bp = 0;
		let lhs = self.parse_expression(min_bp)?;
		self.try_consume(&[RParn])?;

		lhs.unwrap()
	    },
	    // Unary operator support
	    good_tok if good_tok.is_a(Minus) => {
		let ((), right_bp) = Parser::get_prefix_bind_power(&good_tok);
		let Some(rhs) = self.parse_expression(right_bp)?
		else
		{
		    panic!("Unary is missing its operand!");
		};

		ast::Expression::new_cons(good_tok, vec![rhs])
		
	    },

            good_tok => ast::Expression::new_atom(good_tok),
	};

	loop {
	    
	    // Parse operator of expression (if found)
	    let op_kind   = &[Plus, Minus, Div, Mul, Lt, Lte, Gt, Gte, Eq, OrKw, AndKw, AsKw]; 
	    let punc_kind = &[LSBracket, RSBracket,RParn, LBracket, RBracket, Semicolon];
	    let all_kind  = &[&op_kind[..], &punc_kind[..]].concat();
            let op = match self.try_peek(&all_kind)? {
		tok if tok.is_a(EOF) => break,
		tok if tok.is_a(Semicolon) => {self.decrement_parser_pos_by(0); break},
		tok if op_kind.contains(&tok.get_token_kind()) => tok,
		// tok if ty_kind.contains(&tok.get_token_kind()) => tok,
		tok if tok.is_a(LSBracket) => tok,
		tok if tok.is_a(RSBracket) => break,
		tok if tok.is_a(RParn) => break,
		tok if tok.is_a(LBracket) => break,
		tok if tok.is_a(RBracket) => break,
		bad_tok => panic!("unsupported token: {:?}", bad_tok),
            };

	    // Handle postfix
	    if let Some((left_bp, ())) = Parser::get_postfix_bind_power(&op)
	    {
		// To maintain operator precedence
		if left_bp < minimum_bp
		{
		    break;
		}

		self.advance_parser_pos();
		// Parse subscript operator
		lhs = if op.is_a(LSBracket) {
		    let Some(rhs) = self.parse_expression(0)?
		    else
		    {
			panic!("Subscript operator has missing value!");
		    };
		    self.try_consume(&[RSBracket])?;
		    
		    ast::Expression::new_cons(op, vec![lhs, rhs])
		} else {
		    ast::Expression::new_cons(op, vec![lhs])
		};

		continue;
	    }

	    // Handle infix 
	    if let Some((left_bp, right_bp)) = Parser::get_infix_bind_power(&op)
	    {
		// To maintain operator precedence
		if left_bp < minimum_bp
		{
		    break;
		}

		self.advance_parser_pos();
		lhs =  if op.is_a(AsKw) {
		    // Make sure next token is a type
		    self.try_peek(&[IntTy, Ident, FloatTy, BoolTy, TextTy])?;
		    let Some(rhs) = self.parse_expression(right_bp)?
		    else
		    {
			panic!("incomplete binop");
		    };
	
		    ast::Expression::new_cons(op, vec![lhs, rhs])
		    
		} else {

		    let Some(rhs) = self.parse_expression(right_bp)?
		    else
		    {
			panic!("incomplete binop");
		    };

		    ast::Expression::new_cons(op, vec![lhs, rhs])
		};

		continue;

	    }

	    break;
	}

	
	Ok(Some(lhs))
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
            ParserErrorReporter::missing_ty(
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
                // If we have a structure with no fields, just return None
                if consumed_ty_idents.is_empty()
                {
                    return Ok(None);
                }

                // Else, return what we have parsed so far and move on from 
                // structure parsing.
                return Ok(Some(consumed_ty_idents));
            };

            // Make sure we have a seperator between structure fields.
            // We use a `peek` to manually increment parser cursor.
            let curr_tok = self.try_peek(&[Sep, RBracket])?;

            // Store typed ident and go to top of loop to get more typed idents
            if curr_tok.is_a(Sep)
            {
                consumed_ty_idents.push((ident_tok, ty_tok));

                // Advance to next token in stream to parse next struct field
                self.increment_parser_pos_by(1);

                continue 'build_ty_idents;
            }

            // If there is not a comma seperator, stop parsing typed idents.
            // No need to advance parser cursor here since it is done inside the 
            // `self.parse_struct_declaration()` method!
            if curr_tok.is_a(RBracket)
            {
                consumed_ty_idents.push((ident_tok, ty_tok));

                break 'build_ty_idents;
            }
        }

        Ok(Some(consumed_ty_idents))
    }
}
