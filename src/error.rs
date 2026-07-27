use serde_bibtex::token::TokenError;

impl From<TokenError> for Error {
    fn from(value: TokenError) -> Self {
        Self::Token(value)
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Token(TokenError),
    NonAscii,
    InvalidBytes,
    EntryTypeReserved,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
