//! AST module for the Morehead Azalea Compiler
//!
//! NOTE: The formal grammar is defined in the `grammar/` directory inside the file
//! `formal_grammar.pest`.

pub mod errors;
pub mod ast;
pub mod ast_parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
