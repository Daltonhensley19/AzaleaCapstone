//! AST parser error types for the Morehead Azalea Compiler
//!
//! NOTE: The formal grammar is defined in the `grammar/` directory inside the file
//! `formal_grammar.pest`.

use ariadne::{Label, Report, ReportKind, Source};
use thiserror::Error;
use lexer::token::{Token, TokenKind};

// `ParserError` represents a general failure to parse a Azalea program
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Failed to parse Azalea program.")]
    ParseFail,
}

// `ParserErrorReporter` helps with reporting pretty compiler errors for parsing stage
pub struct ParserErrorReporter;

// Specific error report handlers
impl ParserErrorReporter {
    // Error example: `let x <- 5!`
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
    
    pub fn missing_expr_at<'a>(
        at: &str,
        path: &str,
        source: &str,
        offset: usize,
    ) {
        let note = format!(
            "Missing bool expression at {:?}",
            at 
        );
        Report::build(ReportKind::Error, path, offset)
            .with_code(0)
            .with_message("Missing Expression (syntax error)")
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


    // Error example: `let x :: <- 5;`
    pub fn missing_ty<'a>(
        unexpected: &TokenKind,
        expected_toks: &[TokenKind],
        path: &str,
        source: &str,
        offset: usize,
    ) {
        let note = format!(
            "`{0:?}` expected `{1:?}`, but no type was given",
            unexpected, expected_toks
        );
        Report::build(ReportKind::Error, path, offset)
            .with_code(0)
            .with_message("Missing Type (syntax error)")
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

    // Error example: `add_two :: (int int) -> int`
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
    
    // Error example: `let x <- ;`
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

    // Error example: `let x <- 5 + ;`
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

