//! Defines the possible errors that may arise during preprocessing in the
//! Morehead Lambda Compiler.
//!
//! The preprocessor strips multi-line and single-line C-style comments from
//! the source file.

use ariadne::{Config, Label, Report, ReportKind, Source};
use thiserror::Error;

pub struct ErrorReporter;

impl ErrorReporter {
    /// Fancy compiler error that is printed when a multi-line comment is
    /// missing its terminator.
    pub fn missing_terminater(path: &str, source: &str, offset: usize) {
        let note = "`/*` should close with `*/`";
        Report::build(ReportKind::Error, path, offset)
            .with_code(0)
            .with_message("Multi-line comment unclosed")
            .with_label(
                Label::new((path, (offset.saturating_sub(2))..offset))
                    .with_message("Here")
                    .with_color(ariadne::Color::Red),
            )
            .with_note(note)
            .finish()
            .print((path, Source::from(source)))
            .unwrap();
    }

    /// Fancy compiler error that is printed when a bad character is detected
    /// in source file.
    pub fn bad_character(bad_ch: char, path: &str, source: &str, offset: usize) {
        let note = format!("Erroneous character, `{bad_ch:?}`, found in source file.");
        Report::build(ReportKind::Error, path, offset)
            .with_code(0)
            .with_message("Bad/Unsupported character found in source file")
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

#[derive(Clone, Debug, Error)]
/// `PreprocessorError` represents the types of errors that may arise when preprocessing
pub enum PreprocessorError {
    #[error("Failed to preprocess `{0}`")]
    Failed(String),
}
