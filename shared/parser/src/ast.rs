//! AST for the Morehead Azalea Compiler
//!
//! NOTE: The formal grammar is defined in the `grammar/` directory inside the file
//! `formal_grammar.pest`.

use std::fmt;

use lexer::token::Token;
use derive_new::new;
use serde::Serialize;
    
#[derive(Serialize, Debug, new)]
pub struct Type(pub Token);

#[derive(Serialize, Debug, new)]
pub struct Program {
    declarations: Option<Vec<Declaration>>,
}

#[derive(Serialize, Debug, new)]
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

#[derive(Serialize, Debug, new)]
pub struct FuncSignature {
    func_name: Token,
    ty_list: Option<Vec<Token>>,
    ty_ret: Option<Token>,
}

#[derive(Serialize, Debug, new)]
pub struct FuncDefinition {
    func_name: Token,
    arg_list: Option<Vec<Token>>,
    block: Block,
}

#[derive(Serialize, Debug, new)]
pub struct Block {
    statements: Option<Vec<Statement>>,
    expression: Option<Expression>,
}

#[derive(Serialize, Debug, new)]
pub enum RValue {
    Expr(Option<Expression>),
    List(Vec<Option<Expression>>),
    Struct((Token, Vec<Option<Expression>>))
}

#[derive(Serialize, Debug, new)]
pub enum Statement {
    VarBindingInit {
        bind_name: Token,
        ty_hint: Option<Type>,

        rhs: RValue,
    },

    VarBindingMut {
        bind_name: Token,
        expr: Expression,
    },

    Selection {
        if_comp: IfComp,
        elif_comp: Option<ElifComp>,
        else_comp: Option<ElseComp>,
    },

    IndefiniteLoop {
        expr: Expression,
        block: Block,
    },

    DefiniteLoop {
        index_name: Token,
        low_bound: Token,
        high_bound: Token,
        block: Block,
    },
}

#[derive(Serialize, Debug, new)]
pub struct IfComp {
    bool_expr: Expression,
    block: Block,
}

#[derive(Serialize, Debug, new)]
pub struct ElifComp {
    bool_expr: Expression,
    block: Block,
}

#[derive(Serialize, Debug, new)]
pub struct ElseComp {
    block: Block,
}

// S-expressions!
#[derive(Serialize, Debug, new, Clone)]
pub enum Expression {
    Atom(Token),
    Cons(Token, Vec<Expression>),
}

// Way to print an `Expression` using println!()
impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self
        {
            Expression::Atom(i) => write!(f, "{}", i),
            Expression::Cons(head, rest) =>
            {
                write!(f, "({}", head)?;
                for s in rest
                {
                    write!(f, " {}", s)?
                }
                write!(f, ")")
            }
        }
    }
}

