//! Implementations of the [`druid::text::Formatter`] trait.

use druid::text::format::{Formatter, Validation, ValidationError};
use druid::text::Selection;

/// A formatter that can display numeric values.
pub struct NumericFormatter;

/// Errors returned by [`NumericFormatter`].
#[derive(Debug, Clone)]
pub enum NumericValidationError {
    Parse(std::num::ParseIntError),
    InvalidChar(char),
}

impl std::fmt::Display for NumericValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NumericValidationError::InvalidChar(c) => write!(f, "Invalid character '{}'", c),
            NumericValidationError::Parse(err) => write!(f, "Parse failed: {}", err),
        }
    }
}

impl std::error::Error for NumericValidationError {}

impl Formatter<u32> for NumericFormatter {
    fn format(&self, value: &u32) -> String {
        value.to_string()
    }

    fn format_for_editing(&self, value: &u32) -> String {
        value.to_string()
    }

    fn value(&self, input: &str) -> Result<u32, ValidationError> {
        input
            .parse::<u32>()
            .map_err(|err| ValidationError::new(NumericValidationError::Parse(err)))
    }

    fn validate_partial_input(&self, input: &str, _sel: &Selection) -> Validation {
        if input.is_empty() {
            return Validation::success();
        }

        let mut char_iter = input.chars();
        if let Some(c) = char_iter.next() {
            if !(c.is_ascii_digit()) {
                return Validation::failure(NumericValidationError::InvalidChar(c));
            }
        }
        let mut char_iter = char_iter.skip_while(|c| c.is_ascii_digit());
        match char_iter.next() {
            None => return Validation::success(),
            Some(c) => return Validation::failure(NumericValidationError::InvalidChar(c)),
        };
    }
}
