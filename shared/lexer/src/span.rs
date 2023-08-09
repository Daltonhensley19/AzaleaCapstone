//! Helpful API that helps with keeping track of line and column numbers.
//!
//! Spans allow us to point to specific locations in a Azalea source file.

use std::ops::Index;

use serde::Serialize;

#[derive(Serialize, Default, Debug, Clone, Copy)]
/// `SpanPoint` represents an individual point within the range of a `Span`
/// Specifically, you can use `SpanPoint` to get the
pub struct SpanPoint {
    line_num: usize,
    col_num: usize,
    val: char,
}

impl SpanPoint {
    pub fn new(line_num: usize, col_num: usize, val: char) -> Self {
        Self {
            col_num,
            line_num,
            val,
        }
    }
}

impl SpanPoint {
    pub fn incre_col_num(&mut self) {
        self.col_num += 1;
    }

    pub fn incre_line_num(&mut self) {
        self.line_num += 1;
    }

    pub fn get_col_num(&self) -> usize {
        self.col_num
    }

    pub fn get_line_num(&self) -> usize {
        self.line_num
    }
}

#[derive(Default, Debug, Clone)]
pub struct Span {
    points: Vec<SpanPoint>,
}

impl std::fmt::Display for SpanPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0}:{1}", self.line_num, self.col_num)
    }
}

impl Span {
    pub fn new<P: AsRef<str>>(file_content: P) -> Self {
        // Naturally, we start the span in the source file at (1, 1)
        let (mut line_num, mut col_num) = (1, 1);

        // Build `Span` from the chars of the source file
        let mut span = Vec::new();
        for ch in file_content.as_ref().chars()
        {
            if ch.is_ascii()
            {
                // Save `SpanPoint`
                span.push(SpanPoint::new(line_num, col_num, ch));

                // Update `Span` position via line and column number.
                // If we hit a newline, move span to the next line.
                if ch == '\n'
                {
                    line_num += 1;
                    col_num = 1;
                }
                // Otherwise, just increment the span up in the current line
                else
                {
                    col_num += 1;
                }
            }
        }

        Span { points: span }
    }
}

/// Operator overload to index a `Span` to retrieve a `SpanPoint`
impl Index<usize> for Span {
    type Output = SpanPoint;

    fn index(&self, index: usize) -> &Self::Output {
        &self.points[index]
    }
}
