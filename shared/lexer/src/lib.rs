#![allow(unused)]

pub mod errors;
pub mod lexer;
pub mod span;
pub mod token;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works()  {
        let content = include_str!("../source_test.txt");

        let mut lexer = lexer::Lexer::new("source_test.txt", content);

        let tokens = lexer.lex().unwrap();

        println!("{tokens:#?}");
    }
}
