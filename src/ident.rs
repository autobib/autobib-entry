//! # Entry data identifiers
//!
//! There are three types of entry data identifiers:
//!
//! - [`EntryType`]: the entry type, like `article`
//! - [`FieldKey`]: a field key, like `author`
//! - [`FieldValue`]: a field value
//!
//! These are owned types, wrapping an internal string buffer.
//! For the corresponding borrowed types, use:
//!
//! - [`EntryTypeRef`]
//! - [`FieldKeyRef`]
//! - [`FieldValueRef`]
//!
//! An [`EntryType`] has some standard names, enumerated in [`StandardEntryType`].
//! Similarly, a [`FieldKey`] has some standard names, enumerated in [`StandardFieldKey`].
//! Infallible conversion from the standard types, and fallible conversions to standard types, is
//! possible.
//!
//! Standard types can also be parsed directly from strings, which can avoid unnecssary validity
//! checks.
mod deserialize;
mod standard;

use serde::Serialize;
use serde_bibtex::token::{TokenError, check_balanced};

use crate::error::DataError;
pub use standard::{StandardEntryType, StandardFieldKey};

/// A validated entry type (e.g. "article" in `@article{...}`) which satisfies the following
/// requirements:
///
/// 1. composed only of ASCII printable characters with `{}(),= \t\n\\#%\"` and
///    `A..=Z` removed.
/// 2. is not one of `comment`, `preamble`, or `string` (case-insensitive)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct EntryType(pub(crate) String);
impl Default for EntryType {
    fn default() -> Self {
        StandardEntryType::default().into()
    }
}

impl EntryType {
    /// Validate that the given string is a valid entry type string.
    #[inline]
    pub fn validate(s: &str) -> Result<(), DataError> {
        // Condition 1
        validate_ascii_identifier(s.as_bytes())?;

        // Condition 2
        if matches!(s, "comment" | "preamble" | "string") {
            return Err(DataError::EntryTypeReserved);
        }

        Ok(())
    }

    /// Construct a new entry type.
    #[inline]
    pub fn new(mut s: String) -> Result<Self, DataError> {
        s.make_ascii_lowercase();
        Self::validate(&s)?;
        Ok(Self(s))
    }

    /// Construct an entry type from a standard entry type.
    #[inline]
    pub fn standard(et: StandardEntryType) -> Self {
        et.into()
    }

    /// Converts this entry type to a standard entry type, if possible.
    #[inline]
    pub fn as_standard(&self) -> Option<StandardEntryType> {
        StandardEntryType::from_name(&self.0)
    }

    /// Returns if this entry type is standard.
    #[inline]
    pub fn is_standard(&self) -> bool {
        StandardEntryType::is_name(&self.0)
    }
}

impl<'a> EntryTypeRef<'a> {
    /// Construct an entry type from a standard entry type.
    #[inline]
    pub fn standard(et: StandardEntryType) -> Self {
        et.into()
    }

    /// Converts this entry type to a standard entry type, if possible.
    #[inline]
    pub fn as_standard(&self) -> Option<StandardEntryType> {
        StandardEntryType::from_name(self.0)
    }

    /// Returns if this entry type is standard.
    #[inline]
    pub fn is_standard(&self) -> bool {
        StandardEntryType::is_name(self.0)
    }
}

/// A validated field key (e.g. `author` in `...author = {...}`) which satisfies the following
/// requirements:
///
/// 1. composed only of ASCII printable characters with `{}(),= \t\n\\#%\"` and
///    `A..=Z` removed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FieldKey(pub(crate) String);

impl FieldKey {
    /// Validate that the given string is valid as a field key.
    #[inline]
    pub fn validate(s: &str) -> Result<(), DataError> {
        validate_ascii_identifier(s.as_bytes())?;
        Ok(())
    }

    /// Construct a new field key.
    #[inline]
    pub fn new(mut s: String) -> Result<Self, DataError> {
        s.make_ascii_lowercase();
        Self::validate(&s)?;
        Ok(Self(s))
    }

    /// Construct a field key from a standard field key.
    #[inline]
    pub fn standard(et: StandardFieldKey) -> Self {
        et.into()
    }

    /// Converts this field key to a standard field key, if possible.
    #[inline]
    pub fn as_standard(&self) -> Option<StandardFieldKey> {
        StandardFieldKey::from_name(&self.0)
    }

    /// Returns if this field key is standard.
    #[inline]
    pub fn is_standard(&self) -> bool {
        StandardFieldKey::is_name(&self.0)
    }
}

impl<'a> FieldKeyRef<'a> {
    /// Construct a field key from a standard field key.
    #[inline]
    pub fn standard(et: StandardFieldKey) -> Self {
        et.into()
    }

    /// Converts this entry type to a standard field key, if possible.
    #[inline]
    pub fn as_standard(&self) -> Option<StandardFieldKey> {
        StandardFieldKey::from_name(self.0)
    }

    /// Returns if this field key is standard.
    #[inline]
    pub fn is_standard(&self) -> bool {
        StandardFieldKey::is_name(self.0)
    }
}

/// A validated field value (e.g. `John Doe` in `...author = {John Doe}`) which satisfies the
/// following requirements:
///
/// 1. satisfies the balanced `{}` rule (from [`serde_bibtex::token::is_balanced`]).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FieldValue(pub(crate) String);

impl FieldValue {
    /// Validate that the given string is valid as a field value.
    #[inline]
    pub fn validate(s: &str) -> Result<(), DataError> {
        check_balanced(s.as_bytes()).map_err(From::from)
    }

    /// Construct a new field value.
    #[inline]
    pub fn new(s: String) -> Result<Self, DataError> {
        Self::validate(&s)?;
        Ok(Self(s))
    }
}

macro_rules! identifier_impl {
    ($e:ident, $r:ident) => {
        /// A borrowed variant of the corresponding identifier.
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Clone, Copy)]
        pub struct $r<'r>(pub(crate) &'r str);

        impl ::std::convert::From<$r<'_>> for $e {
            fn from(value: $r<'_>) -> Self {
                Self(value.0.into())
            }
        }

        impl<'r> $r<'r> {
            /// Construct a new borrowed variant by wrapping a string slice.
            pub fn new(s: &'r str) -> Result<Self, DataError> {
                $e::validate(s)?;
                Ok(Self(s))
            }

            /// Obtain the inner string slice.
            pub fn inner(&self) -> &'r str {
                &self.0
            }
        }

        impl $e {
            /// Obtain a borrowed variant referencing the internal string buffer.
            pub fn by_ref(&self) -> $r<'_> {
                $r(&self.0)
            }
        }

        impl<'a> AsRef<str> for $r<'a> {
            fn as_ref(&self) -> &'a str {
                &self.0
            }
        }

        impl ::std::fmt::Display for $r<'_> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.0)
            }
        }

        impl ::std::convert::AsRef<str> for $e {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl ::std::fmt::Display for $e {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl ::std::borrow::Borrow<str> for $r<'_> {
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        impl ::std::borrow::Borrow<str> for $e {
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        impl ::std::borrow::Borrow<String> for $e {
            fn borrow(&self) -> &String {
                &self.0
            }
        }

        impl ::std::cmp::PartialEq<str> for $e {
            fn eq(&self, other: &str) -> bool {
                self.0.eq(other)
            }
        }

        impl ::std::cmp::PartialEq<$r<'_>> for $e {
            fn eq(&self, other: &$r<'_>) -> bool {
                self.0.eq(other.0)
            }
        }

        impl ::std::cmp::PartialEq<str> for $r<'_> {
            fn eq(&self, other: &str) -> bool {
                self.0.eq(other)
            }
        }

        impl ::std::cmp::PartialEq<$e> for $r<'_> {
            fn eq(&self, other: &$e) -> bool {
                self.0.eq(&other.0)
            }
        }

        impl ::std::str::FromStr for $e {
            type Err = DataError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s.into())
            }
        }
    };
}

identifier_impl!(EntryType, EntryTypeRef);
identifier_impl!(FieldKey, FieldKeyRef);
identifier_impl!(FieldValue, FieldValueRef);

// the field key requirements are stricted than the field value requirements
impl From<FieldKey> for FieldValue {
    fn from(value: FieldKey) -> Self {
        Self(value.0)
    }
}

impl<'a> From<FieldKeyRef<'a>> for FieldValueRef<'a> {
    fn from(value: FieldKeyRef<'a>) -> Self {
        Self(value.0)
    }
}

/// Lookup table for bytes which could appear in an ASCII entry key or field key.
/// This is precisely the ASCII printable characters with `{}(),= \t\n\\#%\"` and
/// `A..=Z` removed.
static ASCII_IDENTIFIER_ALLOWED: [bool; 256] = {
    const PR: bool = false; // disallowed printable bytes
    const CT: bool = false; // non-printable ascii
    const NA: bool = false; // not ascii
    const UC: bool = false; // uppercase alpha
    const __: bool = true; // permitted bytes
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
        CT, __, PR, PR, __, PR, __, __, PR, PR, __, __, PR, __, __, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, __, PR, __, __, // 3
        __, UC, UC, UC, UC, UC, UC, UC, UC, UC, UC, UC, UC, UC, UC, UC, // 4
        UC, UC, UC, UC, UC, UC, UC, UC, UC, UC, UC, __, PR, __, __, __, // 5
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, PR, __, PR, __, CT, // 7
        NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, // 8
        NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, // 9
        NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, // A
        NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, // B
        NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, // C
        NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, // D
        NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, // E
        NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, NA, // F
    ]
};

/// Check that an identifier is valid ASCII.
#[inline]
pub(crate) fn validate_ascii_identifier(s: &[u8]) -> Result<&str, DataError> {
    if s.is_empty() {
        return Err(DataError::Token(TokenError::Empty));
    }

    match s.iter().find(|&b| !ASCII_IDENTIFIER_ALLOWED[*b as usize]) {
        Some(b) => match char::try_from(*b as u32) {
            Ok(ch) => Err(DataError::Token(TokenError::InvalidChar(ch))),
            Err(_) => Err(DataError::NonAscii),
        },
        // SAFETY: the only bytes permitted by ASCII_IDENTIFIER_ALLOWED are valid ASCII
        None => Ok(unsafe { std::str::from_utf8_unchecked(s) }),
    }
}
