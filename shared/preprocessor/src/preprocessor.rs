//! Defines the preprocessor for the Morehead Lambda Compiler.
//!
//! The preprocessor is responsible for stripping out C-style comments before
//! sending the source file off to the lexer to be tokenized.

use crate::errors::{ErrorReporter, PreprocessorError};

// NOTE: In the future, `content` should be Vec<String> to process many files?
// NOTE: In the future, `path` should be Vec<String> to process many files paths?
#[derive(Default)]
/// `Preprocessor` represents an API for cleaning up the source files to be
/// processed by the compiler.
pub struct Preprocessor {
    content: String,
    path: String,
}

/// CTOR for the `Preprocessor`
impl Preprocessor {
    pub fn new(content: String, path: &str) -> Self {
        Self {
            content,
            path: path.to_owned(),
        }
    }
}

/// Handlers to remove single-line and multi-line comments from source file
impl Preprocessor {
    pub fn remove_singleline_comments(mut self) -> Self {
        // Binding for readability
        let content = &self.content;

        // Create `result` which will be the fixed-up String with no single-line comments
        let mut result = String::with_capacity(content.len());

        // Loop to remove single line comments by appending non-comments to `result` and skipping
        // commented characters
        let mut chars = content.chars().peekable();
        while let Some(ch) = chars.next()
        {
            // Then we are in a single-line comment, so skip passed characters until we are not
            if ch == '/' && chars.peek() == Some(&'/')
            {
                while chars.peek() != Some(&'\n')
                {
                    // skip character
                    chars.next();
                }

                // Maintain the newline
                result.push('\n');
                chars.next();
            }
            else
            {
                result.push(ch);
            }
        }

        // Update result
        self.content = result;

        // Return Self to allow method chaining
        self
    }

    pub fn remove_multiline_comment(mut self) -> Result<Self, PreprocessorError> {
        // Binding for readability
        let content = &self.content;

        // Create `result` which will be the fixed-up String with no multi-line comments
        let mut result = String::with_capacity(content.len());

        // Loop to remove multi-line comments by appending non-comments to `result` and skipping
        // commented characters
        let mut chars = content.chars().peekable();
        //let mut in_multi_comment = false;
        let mut offset_in_file = 0;
        while let Some(ch) = chars.next()
        {
            offset_in_file += 1;

            // Then we are in a multi-line comment, so skip passed characters until we are not
            if ch == '/' && chars.peek() == Some(&'*')
            {
                // Skip until we reach the end of the file or multi-line comment
                chars.next();
                offset_in_file += 1;

                // Save the current offset which marks start of comment
                let comment_offset = offset_in_file;

                // NOTE: Temp_chr is used to both advance the position in the file as well as see if we
                // are handling a whitespace character. If whitespace, then append to `result` to
                // preserve token placement in the file.
                let mut temp_chr = chars.next();
                while temp_chr != Some('*') && chars.peek() != Some(&'/')
                {
                    // If we are trying to handle multi-line comment and we reach the end of the
                    // file, then that means the user forgot to terminate their comment! So, the
                    // comment will actually have no end! Just report as an error.
                    if chars.peek() == None
                    {
                        // Print pretty compiler error
                        ErrorReporter::missing_terminater(
                            self.path.as_ref(),
                            self.content.as_ref(),
                            comment_offset,
                        );

                        // Just eject into application program of the compiler to abort with error
                        return Err(PreprocessorError::Failed(format!(
                            "{path}",
                            path = self.path
                        )));
                    }
                    offset_in_file += 1;

                    // Preserve whitespace to maintain span locations
                    if temp_chr.is_some_and(char::is_whitespace)
                    {
                        result.push(temp_chr.unwrap());
                    }

                    // Advance to the next character
                    temp_chr = chars.next();
                }

                chars.next();
                offset_in_file += 1;
            }
            else
            {
                result.push(ch);
            }
        }

        // Update result
        self.content = result;

        // Return Self to allow method chaining
        Ok(self)
    }

    pub fn normalize_to_ascii(self) -> Result<Self, PreprocessorError> {
        // Binding for readability
        let content = &self.content;

        const VALID_PUNC: &[&str] = &[
            ";", ":", "_", ",", "(", ")", "{", "}", "+", "-", "*", "/", "%", "&", "|", "=", "<",
            ">", "!",
        ];
        const VALID_CONTROL: &[&str] = &["\n", "\t", "\r"];

        // Loop through chars to see if any bad characters are in the source
        // file.
        let chars = content.chars().peekable();
        for (file_offset, ch) in chars.enumerate()
        {
            if !VALID_CONTROL.contains(&ch.to_string().as_str())
                && !ch.is_alphanumeric()
                && !ch.is_whitespace()
                && !VALID_PUNC.contains(&ch.to_string().as_str())
            {
                // Print pretty compiler error
                ErrorReporter::bad_character(
                    ch,
                    self.path.as_ref(),
                    self.content.as_ref(),
                    file_offset,
                );

                // Just bail out of preprocessor
                return Err(PreprocessorError::Failed(format!(
                    "{path}",
                    path = self.path
                )));
            }
        }

        Ok(self)
    }

    /// Gets the cleaned up version of the source file.
    /// NOTE: Make sure you actually clean the file(s) first!
    pub fn get_cleaned_sources(mut self) -> String {
        std::mem::replace(&mut self.content, String::new())
    }
}
