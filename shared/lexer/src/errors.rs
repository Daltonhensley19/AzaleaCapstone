use std::path::Path;

use crate::span::SpanPoint;
use ariadne::{Label, Report, ReportKind, Source};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum LexError {
    #[error("Failed to tokenize {0}")]
    Failed(String)
}


pub struct LexerErrorReporter;

// Specific error report handlers
impl LexerErrorReporter {
    pub fn unsupported_char<'a>(ch: char, path: &str, source: &str, offset: usize) {
        let note = format!("`{0}` is an unsupported character", ch);
        Report::build(ReportKind::Error, path, offset)
            .with_code(0)
            .with_message("Unsupported Character")
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

    pub fn incomplete_tqal<'a>(ch: char, path: &str, source: &str, offset: usize) {
        let note = format!("`{0}` should be `::`", ch);
        Report::build(ReportKind::Error, path, offset)
            .with_code(1)
            .with_message("Type Qualifier is incomplete")
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

    pub fn invalid_ident<'a>(ch: char, path: &str, source: &str, offset: usize) {
        let note = format!(
            "`{0}` should not be attached to the start of a identifier",
            ch
        );
        Report::build(ReportKind::Error, path, offset)
            .with_code(2)
            .with_message("Invalid identifier")
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

    pub fn misplaced_underscore<'a>(ch: char, path: &str, source: &str, offset: usize) {
        let note = "Underscores must come directly before or after letters or other underscores";
        Report::build(ReportKind::Error, path, offset)
            .with_code(3)
            .with_message("Misplaced underscore")
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

    pub fn invalid_float<'a>(ch: char, path: &str, source: &str, offset: usize) {
        let note = "Floats must contain strictly numbers before and after the `.`";
        Report::build(ReportKind::Error, path, offset)
            .with_code(4)
            .with_message("Invalid float")
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
