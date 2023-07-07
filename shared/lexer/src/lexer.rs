//! Defines the lexer (also know as a "scanner") for the Morehead Lambda Compiler.
//!
//! The lexer, using a DFA, converts raw characters into meaningful
//! words and punctuation (tokens).

use crate::errors::{LexError, LexerErrorReporter};
use crate::span::{Span, SpanPoint};
use crate::token::{Token, TokenHint, TokenKind};
use ariadne::Report;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Default, Debug)]
pub struct Lexer {
    /// Token that we are currently trying to build
    current_tok: String,

    /// Position in the source file
    current_pos: Span,

    /// Path to the source file
    source_path: PathBuf,

    /// Content of the source file
    source_content: String,

    /// Lexer index into the file
    index: usize,

    /// Flag to say we are at the end of the source file
    eof: bool,

    /// Token we suspect will be built
    hint_tok: TokenHint,

    /// Flag to keep track of if we hit an error
    found_error: bool,
}

/// Constructor for the `Lexer`
impl Lexer {
    pub fn new<S, P>(file_path: P, file_content: S) -> Self
    where
        S: AsRef<str>,
        P: AsRef<Path>,
    {
        Lexer {
            current_tok: String::new(),
            current_pos: Span::new(&file_content),
            source_path: file_path.as_ref().to_path_buf(),
            source_content: file_content.as_ref().to_string(),
            index: 0,
            eof: false,
            hint_tok: TokenHint::Undetermined,
            found_error: false,
        }
    }
}

/// General getters/setters/incrementers
impl Lexer {
    /// Get the backing file index as a `usize`
    ///
    /// # Returns
    ///
    /// Returns the backing file index as a `usize`
    ///
    fn get_file_index(&self) -> usize {
        self.index
    }

    /// Increments the file index by one
    ///
    fn incre_file_index(&mut self) {
        self.index += 1;
    }

    /// Get the backing `SpanPoint` of the current file index
    ///
    /// # Returns
    ///
    /// Returns current position in file as a `SpanPoint`
    ///
    fn get_current_span_pos(&self) -> SpanPoint {
        let file_index = self.get_file_index();
        self.current_pos[file_index]
    }

    /// Get the start and end of a span. Useful to get the span of a `Token`
    ///
    /// # Parameters
    ///
    /// * `offset`: subtracted from index to get the start of `Token` span
    ///
    /// # Returns
    ///
    /// Returns `(span_start, span_end)`
    ///
    fn get_span_start_and_end_with_offset(&self, offset: usize) -> (SpanPoint, SpanPoint) {
        let file_index = self.get_file_index();

        let start = self.current_pos[file_index.saturating_sub(offset)];
        let end = self.current_pos[file_index.saturating_sub(1)];

        (start, end)
    }

    /// Increment the current file index by some offset, but clamp in the event of overflowing file
    /// length.
    ///
    /// # Parameters
    ///
    /// * `incre`: amount to increment by.
    ///
    fn incre_file_index_by(&mut self, incre: usize) {
        // Advance to next index in file
        self.index += incre;

        // Clamp to the file if we go passed EOF to prevent out-of-bounds access
        if self.index >= self.source_content.len()
        {
            self.eof = true;
            self.index = self.source_content.len();
        }
    }
}

/// Peek implementations for `Lexer`
impl Lexer {
    /// Get the character in the next file index position
    ///
    /// # Returns
    ///
    /// Returns `Some` if we `peek` and find a char, `None` otherwise.
    fn peek(&self) -> Option<char> {
        // Get next char based on the lexer's position in the source file plus one
        let file_index = self.get_file_index() + 1;
        let next_char = self.source_content.chars().nth(file_index);

        next_char
    }

    /// Get the character in the current file index position
    ///
    /// # Returns
    ///
    /// Returns `Some` if we `peek_current` and find a char, `None` otherwise.
    fn peek_current(&mut self) -> Option<char> {
        // Get current char based on the lexer's position in the source file plus one
        let file_index = self.get_file_index();
        let current_char = self.source_content.chars().nth(file_index);

        // Make sure we throw an error if we detect invalid chars
        if current_char.is_some_and(|ch| !ch.is_ascii())
        {
            // Generate error report and print
            LexerErrorReporter::unsupported_char(
                current_char.unwrap(),
                self.source_path.to_str().unwrap(),
                &self.source_content,
                self.get_file_index(),
            );

            // Set flag but continue attempting to lex to find more errors
            self.found_error = true;

            return None;
        }

        current_char
    }

    /// Get the character in the previous file index position
    ///
    /// # Returns
    ///
    /// Returns `Some` if we `peek_previous` and find a char, `None` otherwise.
    fn peek_previous(&self) -> Option<char> {
        // Get previous char based on the lexer's position in the source file minus one
        let file_index = self.get_file_index().saturating_sub(1);
        let current_char = self.source_content.chars().nth(file_index);

        current_char
    }
}

/// Consume implementations for punctuation
impl Lexer {
    /// Create a `Token` that is one char in length.
    ///
    /// # Parameters
    ///
    /// * `ch`: character that we are trying to consume to `Token`.
    /// * `kind`: the variant of `Token` that we have made.
    ///
    /// # Returns
    ///
    /// Returns `Some` if we make a one-char length `Token`, `None` otherwise.
    pub fn consume_one_chars(&mut self, ch: char, kind: TokenKind) -> Option<Token> {
        // Move to next position in file and get start and end span of current token
        self.incre_file_index_by(1);
        let (span_start, span_end) = self.get_span_start_and_end_with_offset(1);
        let file_index = self.get_file_index();
        let is_reserved = false;

        Some(Token::new(
            ch.to_string(),
            kind,
            span_start,
            span_end,
            file_index,
            is_reserved,
        ))
    }

    /// Create a `Token` that is one or two chars in length based on calling `peek()`.
    ///
    /// # Parameters
    ///
    /// * `ch1`: character that we are trying to consume to `Token`.
    /// * `ch2`: character that we are trying to consume to `Token`.
    /// * `kind1`: the variant of `Token` that we have made in the event of 1-char `Token`.
    /// * `kind2`: the variant of `Token` that we have made in the event of 2-char `Token`.
    ///
    /// # Returns
    ///
    /// Returns `Some` if we make a 1-char or 2-char length `Token`, `None` otherwise.
    fn consume_one_or_two_chars(
        &mut self,
        ch1: char,
        ch2: char,
        kind1: TokenKind,
        kind2: TokenKind,
    ) -> Option<Token> {
        let next_char = self.peek().expect("Tried to peek pass EOF.");

        if next_char == ch2
        {
            // Move to next position in file and get start and end span of current token
            self.incre_file_index_by(2);
            let (span_start, span_end) = self.get_span_start_and_end_with_offset(2);
            let file_index = self.get_file_index();

            return Some(Token::new(
                format!("{0}{1}", ch1, ch2),
                kind2,
                span_start,
                span_end,
                file_index,
                false,
            ));
        }

        // Move to next position in file and get start and end span of current token
        self.incre_file_index_by(1);
        let (span_start, span_end) = self.get_span_start_and_end_with_offset(1);
        let file_index = self.get_file_index();
        let is_reserved = false;

        Some(Token::new(
            ch1.to_string(),
            kind1,
            span_start,
            span_end,
            file_index,
            is_reserved,
        ))
    }
}

// Consume implementations for complex tokens
impl Lexer {
    /// Create a `Token` that is a int or a float
    ///
    /// # Parameters
    ///
    /// * `token_buffer`: the accumulated `Token` buffer that the `Lexer` uses
    ///
    fn consume_num_or_float_lit(&mut self, token_buffer: &mut Vec<Token>) {
        // Determine if we have a int literal of float
        let token_kind = if self.current_tok.contains('.')
        {
            TokenKind::FloatLit
        }
        else
        {
            TokenKind::NumLit
        };

        // Get start and end of the current token's span
        let (span_start, span_end) =
            self.get_span_start_and_end_with_offset(self.current_tok.len());

        let file_index = self.get_file_index();

        // Create the `Token`
        let raw_token_content = self.current_tok.clone();
        let is_reserved = false;
        let tok = Token::new(
            raw_token_content,
            token_kind,
            span_start,
            span_end,
            file_index,
            is_reserved,
        );

        // Push to the internal `Token` buffer
        token_buffer.push(tok);

        // Reset hint
        self.hint_tok = TokenHint::Undetermined;

        // Clear buffer for next token
        self.current_tok.clear();
    }

    /// Create a `Token` that is a ident or reserved keyword
    ///
    /// # Parameters
    ///
    /// * `token_buffer`: the accumulated `Token` buffer that the `Lexer` uses
    ///
    fn consume_ident_or_reserved(&mut self, token_buffer: &mut Vec<Token>) {
        // See if we can refine to a more specific token other then `Ident`
        let (token_kind, is_reserved) = TokenKind::refined_or_ident(&self.current_tok);

        // Get start and end of the current token's span
        let (span_start, span_end) =
            self.get_span_start_and_end_with_offset(self.current_tok.len());

        let file_index = self.get_file_index();

        // Create the `Token`
        let raw_token_content = self.current_tok.clone();
        let tok = Token::new(
            raw_token_content,
            token_kind,
            span_start,
            span_end,
            file_index,
            is_reserved,
        );

        // Push to the internal `Token` buffer
        token_buffer.push(tok);

        // Reset hint
        self.hint_tok = TokenHint::Undetermined;

        // Clear buffer for next token
        self.current_tok.clear();
    }
}

impl Lexer {
    fn lex_punctuation(&mut self) -> Option<Token> {
        // Get the current character and move to next character if we find an issue
        let Some(character) = self.peek_current()
        else
        {
            self.incre_file_index_by(1);
            return None;
        };

        match character
        {
            // "Easy" chars to consume
            '+' => self.consume_one_chars('+', TokenKind::Plus),
            '*' => self.consume_one_chars('*', TokenKind::Mul),
            '/' => self.consume_one_chars('/', TokenKind::Div),
            '-' => self.consume_one_or_two_chars('-', '>', TokenKind::Minus, TokenKind::RetArrow),
            '>' => self.consume_one_or_two_chars('>', '=', TokenKind::Gt, TokenKind::Gte),
            '=' => self.consume_one_or_two_chars('=', '=', TokenKind::FnDef, TokenKind::Eq),
            ';' => self.consume_one_chars(';', TokenKind::Semicolon),
            '(' => self.consume_one_chars('(', TokenKind::LParn),
            ')' => self.consume_one_chars(')', TokenKind::RParn),
            '{' => self.consume_one_chars('{', TokenKind::LBracket),
            '}' => self.consume_one_chars('}', TokenKind::RBracket),
            ',' => self.consume_one_chars(',', TokenKind::Sep),
            // Special case
            '<' =>
            {
                let next_char = self.peek().expect("Tried to peek pass EOF.");

                if next_char == '-'
                {
                    // Move to next position in file and get start and end span of current token
                    self.incre_file_index_by(2);
                    let (span_start, span_end) = self.get_span_start_and_end_with_offset(2);
                    let file_index = self.get_file_index();
                    let is_reserved = false;

                    return Some(Token::new(
                        "<-".to_string(),
                        TokenKind::Assign,
                        span_start,
                        span_end,
                        file_index,
                        is_reserved,
                    ));
                }
                else if next_char == '='
                {
                    // Move to next position in file and get start and end span of current token
                    self.incre_file_index_by(2);
                    let (span_start, span_end) = self.get_span_start_and_end_with_offset(2);
                    let file_index = self.get_file_index();
                    let is_reserved = false;

                    return Some(Token::new(
                        "<=".to_string(),
                        TokenKind::Lte,
                        span_start,
                        span_end,
                        file_index,
                        is_reserved,
                    ));
                }
                else
                {
                    // Move to next position in file and get start and end span of current token
                    self.incre_file_index_by(1);
                    let (span_start, span_end) = self.get_span_start_and_end_with_offset(1);
                    let file_index = self.get_file_index();
                    let is_reserved = false;

                    return Some(Token::new(
                        "<".to_string(),
                        TokenKind::Lt,
                        span_start,
                        span_end,
                        file_index,
                        is_reserved,
                    ));
                }
            }
            // Special case
            ':' =>
            {
                let next_char = self.peek().expect("Tried to peek pass EOF.");

                if next_char == ':'
                {
                    // Move to next position in file and get start and end span of current token
                    self.incre_file_index_by(2);
                    let (span_start, span_end) = self.get_span_start_and_end_with_offset(2);
                    let file_index = self.get_file_index();
                    let is_reserved = false;

                    return Some(Token::new(
                        "::".to_string(),
                        TokenKind::TQualifer,
                        span_start,
                        span_end,
                        file_index,
                        is_reserved,
                    ));
                }
                else
                {
                    // error report
                    LexerErrorReporter::incomplete_tqal(
                        ':',
                        self.source_path.to_str().unwrap(),
                        &self.source_content,
                        self.get_file_index(),
                    );

                    // Proceed to next character and attempt to find other errors
                    self.found_error = true;
                    None
                }
            }
            // Special case
            '.' =>
            {
                // We have a `RecordDot` token if the previous and next character are between a `.`
                if self
                    .peek_previous()
                    .is_some_and(|c0| c0.is_alphabetic() || c0 == '_')
                    && self
                        .peek()
                        .is_some_and(|c2| c2.is_alphabetic() || c2 == '_')
                {
                    // Move to next position in file and get start and end span of current token
                    self.incre_file_index_by(1);
                    let (span_start, span_end) = self.get_span_start_and_end_with_offset(1);
                    let file_index = self.get_file_index();
                    let is_reserved = false;

                    Some(Token::new(
                        ".".to_string(),
                        TokenKind::RecordDot,
                        span_start,
                        span_end,
                        file_index,
                        is_reserved,
                    ))
                }
                // We have a `ExRange` if we observe `..`
                else if self.peek().is_some_and(|c| c == '.')
                {
                    // Move to next position in file and get start and end span of current token
                    self.incre_file_index_by(2);
                    let (span_start, span_end) = self.get_span_start_and_end_with_offset(2);
                    let file_index = self.get_file_index();
                    let is_reserved = false;

                    Some(Token::new(
                        "..".to_string(),
                        TokenKind::ExRange,
                        span_start,
                        span_end,
                        file_index,
                        is_reserved,
                    ))
                }
                else
                {
                    // error report
                    LexerErrorReporter::invalid_float(
                        '.',
                        self.source_path.to_str().unwrap(),
                        &self.source_content,
                        self.get_file_index(),
                    );

                    // Proceed to next character and attempt to find other errors
                    self.found_error = true;
                    None
                }
            }
            // Unknown character found
            ch =>
            {
                // error report
                LexerErrorReporter::unsupported_char(
                    ch,
                    self.source_path.to_str().unwrap(),
                    &self.source_content,
                    self.get_file_index(),
                );

                self.found_error = true;
                None
            }
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, LexError> {
        // Get raw characters from the loaded source file
        let mut file_chars = &self.source_content;

        // Check to see if the file is empty. Just skip while loop if source file is empty.
        let is_empty_file = self.source_content.trim().is_empty();

        // Token building loop. Quit if we error or we reach EOF.
        let mut tokens: Vec<Token> = Vec::new();
        while !self.eof && !is_empty_file
        {
            // Get the current character
            let ch = self
                .peek_current()
                .expect("Peeked passed EOF. (Should be impossible here!)");

            // Where we determine what token we think we are building
            if self.current_tok.trim().is_empty()
            {
                // We have begun building an ident or reserved token
                if ch.is_alphabetic()
                {
                    self.hint_tok = TokenHint::IdentOrKeyword;
                }
                // We have begun building a number-like token
                else if ch.is_numeric()
                {
                    self.hint_tok = TokenHint::Number;
                }
            }

            // Make sure numbers do not come directly before letters
            if ch.is_numeric() && self.peek().expect("peeked passed EOF.").is_alphabetic()
            {
                // error report
                LexerErrorReporter::invalid_ident(
                    ch,
                    self.source_path.to_str().unwrap(),
                    &self.source_content,
                    self.get_file_index(),
                );

                // Proceed to next character and attempt to find other errors
                self.found_error = true;
                self.incre_file_index_by(1);
                continue;
            }

            // If we observe a base-10 digit and we were already making a number, just append an move to
            // next character
            if ch.is_numeric() && self.hint_tok == TokenHint::Number
            {
                // Append to `current_tok` to build number
                self.current_tok.push(ch);
                self.incre_file_index_by(1);

                continue;
            }

            // If we observe an `_` (valid for idents) and we are not making an ident,
            // dump currently built token work on processing '_'
            if ch == '_' && self.hint_tok != TokenHint::IdentOrKeyword
            {
                // Next character following a '_' must be either a letter or another '_' since
                // idents can have underscores in them.
                if self
                    .peek()
                    .is_some_and(|ch| !ch.is_alphabetic() && ch != '_')
                {
                    // error report
                    LexerErrorReporter::misplaced_underscore(
                        ch,
                        self.source_path.to_str().unwrap(),
                        &self.source_content,
                        self.get_file_index(),
                    );

                    // Proceed to next character and attempt to find other errors
                    self.found_error = true;
                    self.incre_file_index_by(1);
                    continue;
                }

                // Dump number token if we were already building one
                if !self.current_tok.trim().is_empty() && self.hint_tok == TokenHint::Number
                {
                    self.consume_num_or_float_lit(&mut tokens);
                }

                // Begin processing '_'
                self.current_tok.push(ch);

                // Set hint
                self.hint_tok = TokenHint::IdentOrKeyword;

                // Advance to next character
                self.incre_file_index_by(1);

                // Begin at top to start next token
                continue;
            }

            // Critcal moment where we see if we hit a whitespace character. This is where we try
            // to "tokenize" the token we have hence been building.
            if ch.is_whitespace()
            {
                // Consume the current ident or reserved token
                if !self.current_tok.trim().is_empty() && self.hint_tok == TokenHint::IdentOrKeyword
                {
                    self.consume_ident_or_reserved(&mut tokens);
                }

                // Consume the current number-like token
                if !self.current_tok.trim().is_empty() && self.hint_tok == TokenHint::Number
                {
                    self.consume_num_or_float_lit(&mut tokens);
                }

                // Start at the top of loop to begin processing next character
                self.incre_file_index_by(1);
                continue;
            }

            // Attempt to lex punctuation
            if ch.is_ascii_punctuation()
            {
                // If anything is currently in the token buffer, dump it out correctly
                let was_building_token = !self.current_tok.trim().is_empty();

                if was_building_token
                {
                    // We must know what kind of token at start of token building
                    assert!(
                        self.hint_tok != TokenHint::Undetermined,
                        "Token hint must be set at this point."
                    );

                    // See if `token_kind` is a ident or keyword
                    let (token_kind, is_reserved) = if self.hint_tok == TokenHint::IdentOrKeyword
                    {
                        // If we were just dealing with a '_' punctuation, then we can continue to top
                        // of loop since its really apart of building idents
                        if ch == '_'
                        {
                            self.current_tok.push(ch);
                            self.incre_file_index_by(1);
                            continue;
                        }

                        // See if we can refine to a more specific token other then `Ident`
                         TokenKind::refined_or_ident(&self.current_tok)
                        
                    }
                    // See if `token_kind` is number-like
                    else
                    {
                        // If we were just dealing with a '.' punctuation, then we can continue to top
                        // of loop since its really apart of building floats
                        if ch == '.'
                            && self.peek_previous().is_some_and(|c0| c0.is_numeric())
                            && self.peek().is_some_and(|c2| c2.is_numeric())
                        {
                            self.current_tok.push(ch);
                            self.incre_file_index_by(1);
                            continue;
                        }

                        // If the currently built `Token` has an `.` in it, then refine type of
                        // number
                        let is_reserved = false;
                        if self.current_tok.contains('.')
                        {
                            (TokenKind::FloatLit, is_reserved)
                        }
                        else
                        {
                            (TokenKind::NumLit, is_reserved)
                        }
                    };

                    // Get start and end of span of current `Token`
                    let (span_start, span_end) =
                        self.get_span_start_and_end_with_offset(self.current_tok.len());

                    let file_index = self.get_file_index();

                    // Make the `Token`
                    let tok = Token::new(
                        self.current_tok.clone(),
                        token_kind,
                        span_start,
                        span_end,
                        file_index,
                        is_reserved
                    );

                    // Push `Token` to internal token buffer
                    tokens.push(tok);

                    // Reset hint
                    self.hint_tok = TokenHint::Undetermined;

                    // Clear buffer for next token
                    self.current_tok.clear();
                }

                // Since the complex tokens have been attempted, try to see if we can progess by
                // lexing the token as just a simple punctuation. If `lex_punctuation` returns
                // `Some`, then this is the case.
                if let Some(tok) = self.lex_punctuation()
                {
                    // Push punctuation and clear buffer for next token
                    tokens.push(tok);
                    self.current_tok.clear();
                    self.hint_tok = TokenHint::Undetermined;

                    // Start at the top of loop to begin processing next character
                    continue;
                }
                else
                {
                    // Just proceed to the next character if we sense a problem and report error
                    self.hint_tok = TokenHint::Undetermined;
                    self.current_tok.clear();
                    self.incre_file_index_by(1);
                    continue;
                }
            }

            // Build complex token character by character
            self.incre_file_index_by(1);
            self.current_tok.push(ch);
        }

        // If we generated zero errors, return the tokens.
        // Otherwise, return Err to avoid giving user incorrect output.
        if !self.found_error
        {
            Ok(tokens)
        }
        else
        {
            // Consumers of this API will probably want to just fail fast
            Err(LexError::Failed(format!(
                "`{path}`",
                path = self.source_path.display()
            )))
        }
    }
}
