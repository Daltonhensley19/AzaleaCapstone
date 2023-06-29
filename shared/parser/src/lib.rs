//! Lambda programing language AST parser
//!
//! NOTE: The formal grammar is defined in the `grammar/` directory inside the file
//! `formal_grammar.pest`.

mod ast {
    use lexer::token::Token;

    #[derive(Debug)]
    struct Program {
        declarations: Option<Vec<Declaration>>,
        main: Main,
    }

    #[derive(Debug)]
    struct Main {
        signature: MainSignature,
        definition: MainDefinition,
    }

    #[derive(Debug)]
    struct MainSignature {
        name: Token,
        ty_list: Option<Vec<Token>>,
        ty_ret: Option<Token>,
    }

    #[derive(Debug)]
    struct MainDefinition {
        name: Token,
        arg_list: Option<Vec<Token>>,
        block: Block,
    }

    #[derive(Debug)]
    enum Declaration {
        Function {
            signature: FuncSignature,
            definition: FuncDefinition,
        },
        // TODO: Add support for enums and structs
        // Choice,
        // Struct
    }

    #[derive(Debug)]
    struct FuncSignature {
        func_name: Token,
        ty_list: Option<Vec<Token>>,
        ty_ret: Option<Token>,
    }

    #[derive(Debug)]
    struct FuncDefinition {
        func_name: Token,
        arg_list: Option<Vec<Token>>,
        block: Block,
    }

    #[derive(Debug)]
    struct Block {
        statements: Option<Vec<Statement>>,
        expression: Option<Expression>,
    }

    #[derive(Debug)]
    enum Statement {
        VarBinding { bind_name: Token, expr: Expression },
    }

    #[derive(Debug)]
    struct Expression {
        term: Term,
        other: Option<Vec<(TermOp, Term)>>,
    }

    #[derive(Debug)]
    struct Term {
        factor: Factor,
        other: Option<Vec<(FactOp, Factor)>>,
    }

    #[derive(Debug)]
    struct Factor {
        atom: Atom,
    }

    #[derive(Debug)]
    enum Atom {
        Token,

        // `(` Expr `)`
        Expression,
    }

    #[derive(Debug)]
    // `+` | `-`
    struct TermOp(Token);

    #[derive(Debug)]
    // `*` | `/`
    struct FactOp(Token);
}

fn main() {
    println!("Hello, world!");
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
