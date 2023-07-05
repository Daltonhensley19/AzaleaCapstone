//! AST parser for the Morehead Lambda Compiler
//!
//! NOTE: The formal grammar is defined in the `grammar/` directory inside the file
//! `formal_grammar.pest`.

use std::cell::Cell;

use lexer::token::{Token, TokenKind};

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

pub struct Parser {
    tokens: Vec<Token>,

    /// `pos` is a `Cell` to help limit mutation and to keep methods of `Parser` only needing
    /// `&self` instead of `&mut self`. This is a controlled form of mutation! 
    pos: Cell<usize>,
}

/// CTOR for the `Parser`
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: Cell::new(0) }
    }
}

/// Internal helper functions to build smaller parsers  
impl Parser {

    /// Get the next upcoming `Token` by reference based on current `Parser` 
    /// index `pos`.
    fn peek(&self) -> &Token {
        let curr_tok_pos = &self.pos;

        &self.tokens[curr_tok_pos.get()]
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

impl Parser {
    pub fn parse(&self) -> ast::Program {
        let declarations = self.parse_declarations();
        let main = self.parse_main();

        ast::Program::new(declarations, main)
    }

    fn parse_declarations(&self) -> Option<Vec<ast::Declaration>> {
        //ast::Declaration::new_function()
        //
        let curr_token = self.peek();
        let declaration = match curr_token {
            token if token.is_a(TokenKind::Ident) => self.parse_function_declaration(),
            // TODO: Add support for `struct` and `choice` decls
           _ => unreachable!()

        };

        todo!()
    }

    fn parse_function_declaration(&self) -> ast::Declaration {
        let function_signature = self.parse_function_signature();
        let function_definition = self.parse_function_definition();

        ast::Declaration::new_function(function_signature, function_definition);

        unimplemented!();
    }

    fn parse_function_signature(&self) -> ast::FuncSignature {
        // Get name of function
        let func_name = self.peek(); 
        self.advance_parser_pos();

        // Get input types of function signature 
        let func_input_tys = if self.peek().is_a(TokenKind::TQualifer) {
            self.advance_parser_pos();
        };
        
        unimplemented!();
        //ast::FuncSignature::new(func_name, )
    }

    fn parse_function_definition(&self) -> ast::FuncDefinition {
        unimplemented!();
    }

    fn parse_main(&self) -> ast::Main {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
