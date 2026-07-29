//! Errors which may occur during type conversion and validation
//!
//! There are two main error types:
//!
//! - [`AccessError`]: errors occuring during zero-copy deserialization.
//! - [`DataError`]: errors occuring while constructing entry data.
use std::{fmt, str::Utf8Error};

use serde_bibtex::token::TokenError;

/// An error which may occur during zero-copy deserialization.
#[derive(Debug, PartialEq)]
pub enum AccessError {
    /// The format is not recognized by this variant.
    Unrecognized,
    /// The header bytes are incomplete or invalid.
    InvalidHeader,
    /// The header bytes are incomplete or invalid.
    InvalidEntryType,
    /// The fields metadata is incomplete.
    IncompleteFields,
    /// There are trailing bites starting at the given byte offset.
    TrailingBytes(usize),
    /// The specified field contains an invalid index.
    InvalidIndex(usize),
    /// The specified field contains an index which points to a byte-offset which is not a char
    /// boundary.
    InvalidStrOffset(usize),
    /// The string data is not valid Utf-8.
    InvalidUtf8(Utf8Error),
}

impl From<Utf8Error> for AccessError {
    fn from(err: Utf8Error) -> Self {
        Self::InvalidUtf8(err)
    }
}

impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unrecognized => f.write_str("data format could not be recognized"),
            Self::InvalidHeader => f.write_str("data has invalid or incomplete header"),
            Self::InvalidEntryType => f.write_str("data has invalid entry type pointer"),
            Self::IncompleteFields => f.write_str("data has incomplete field metadata"),
            Self::TrailingBytes(idx) => write!(
                f,
                "metadata indicates that the data should end at {idx}, but there are still remaining bytes"
            ),
            Self::InvalidIndex(idx) => write!(f, "field {idx} contains an invalid data pointer"),
            Self::InvalidStrOffset(idx) => write!(f, "field {idx} contains a data pointer"),
            Self::InvalidUtf8(utf8_error) => write!(f, "data contains invalid Utf-8: {utf8_error}"),
        }
    }
}

impl std::error::Error for AccessError {}

/// An error which may occur while trying to construct entry data.
#[derive(Debug, PartialEq)]
pub enum DataError {
    /// There was a syntax error in a BibTeX token or identifier.
    Token(TokenError),
    /// An entry type is one of the reserved names `comment`, `preamble`, `string`.
    EntryTypeReserved,
    /// An identifier which was expected to be Ascii is not Ascii
    NonAscii,
    /// Fields are not sorted by field key.
    Unsorted,
}

impl From<TokenError> for DataError {
    fn from(value: TokenError) -> Self {
        Self::Token(value)
    }
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Token(err) => err.fmt(f),
            Self::NonAscii => f.write_str("identifier is not ASCII"),
            Self::Unsorted => f.write_str("fields are not sorted by field key"),
            Self::EntryTypeReserved => {
                f.write_str("entry type must not be a reserved name: comment, preamble, string")
            }
        }
    }
}

impl std::error::Error for DataError {}
