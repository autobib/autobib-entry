use std::{fmt, str::Utf8Error};

use serde_bibtex::token::TokenError;

pub enum DeserializationError {
    IncompleteHeader,
    IncompleteFields,
    TrailingBytes(usize),
    InvalidIndex(usize),
    InvalidStrOffset(usize),
    InvalidUtf8(Utf8Error),
}

impl From<Utf8Error> for DeserializationError {
    fn from(err: Utf8Error) -> Self {
        Self::InvalidUtf8(err)
    }
}

impl fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompleteHeader => f.write_str("data has incomplete header"),
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

#[derive(Debug, PartialEq)]
pub enum DataError {
    Token(TokenError),
    NonAscii,
    InvalidBytes,
    EntryTypeReserved,
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
            Self::InvalidBytes => f.write_str("failed to deserialize from raw bytes"),
            Self::EntryTypeReserved => {
                f.write_str("entry type must not be a reserved name: comment, preamble, string")
            }
        }
    }
}

impl std::error::Error for DataError {}
