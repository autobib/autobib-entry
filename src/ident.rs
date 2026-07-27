use serde_bibtex::token::{TokenError, check_balanced};

use crate::error::DataError;

/// A validated entry type (e.g. "article" in `@article{...}`) which satisfies the following
/// requirements:
///
/// 1. composed only of ASCII printable characters with `{}(),= \t\n\\#%\"` and
///    `A..=Z` removed.
/// 2. is not one of `comment`, `preamble`, or `string` (case-insensitive)
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq, PartialOrd, Ord))]
#[repr(transparent)]
pub struct EntryType(pub(crate) String);

impl Default for EntryType {
    fn default() -> Self {
        Self("misc".into())
    }
}

impl EntryType {
    /// Construct a new entry type.
    #[inline]
    pub fn try_new(mut s: String) -> Result<Self, DataError> {
        s.make_ascii_lowercase();

        // Condition 1
        validate_ascii_identifier(s.as_bytes())?;

        // Condition 2
        if matches!(s.as_str(), "comment" | "preamble" | "string") {
            return Err(DataError::EntryTypeReserved);
        }

        Ok(Self(s))
    }

    pub fn misc() -> Self {
        Self("misc".into())
    }

    pub fn preprint() -> Self {
        Self("preprint".into())
    }

    pub fn book() -> Self {
        Self("book".into())
    }

    pub fn in_collection() -> Self {
        Self("incollection".into())
    }

    pub fn article() -> Self {
        Self("article".into())
    }
}

/// A validated field key (e.g. `author` in `...author = {...}`) which satisfies the following
/// requirements:
///
/// 1. composed only of ASCII printable characters with `{}(),= \t\n\\#%\"` and
///    `A..=Z` removed.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq, PartialOrd, Ord))]
#[repr(transparent)]
pub struct FieldKey(pub(crate) String);

impl FieldKey {
    #[inline]
    pub fn try_new(mut s: String) -> Result<Self, DataError> {
        s.make_ascii_lowercase();

        validate_ascii_identifier(s.as_bytes())?;

        Ok(Self(s))
    }
}

// the field key requirements are stricted than the field value requirements
impl From<FieldKey> for FieldValue {
    fn from(value: FieldKey) -> Self {
        Self(value.0)
    }
}

/// A validated field value (e.g. `John Doe` in `...author = {John Doe}`) which satisfies the
/// following requirements:
///
/// 1. satisfies the balanced `{}` rule (from [`serde_bibtex::token::is_balanced`]).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq, PartialOrd, Ord))]
#[repr(transparent)]
pub struct FieldValue(pub(crate) String);

impl FieldValue {
    #[inline]
    pub fn try_new(s: String) -> Result<Self, DataError> {
        check_balanced(s.as_bytes())?;

        Ok(Self(s))
    }
}

macro_rules! identifier_impl {
    ($e:ident, $r:ident, $a:ident) => {
        impl ::std::borrow::Borrow<str> for $a {
            fn borrow(&self) -> &str {
                self.0.as_str()
            }
        }

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, Clone, Copy)]
        pub struct $r<'r>(pub(crate) &'r str);

        impl From<$r<'_>> for $e {
            fn from(value: $r<'_>) -> Self {
                Self(value.0.into())
            }
        }

        impl<'r> $r<'r> {
            pub fn inner(&self) -> &'r str {
                &self.0
            }
        }

        impl $e {
            pub fn ref_inner(&self) -> $r<'_> {
                $r(&self.0)
            }
        }

        impl $a {
            pub fn ref_inner(&self) -> $r<'_> {
                $r(&self.0.as_str())
            }
        }

        impl AsRef<str> for $e {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl ::std::fmt::Display for $e {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.0.as_ref())
            }
        }

        // Borrow implementation for convenience of using `get.
        impl ::std::borrow::Borrow<str> for $e {
            fn borrow(&self) -> &str {
                self.0.as_ref()
            }
        }

        // Borrow implementation for convenience of using `get.
        impl ::std::borrow::Borrow<String> for $e {
            fn borrow(&self) -> &String {
                &self.0
            }
        }

        impl PartialEq<str> for $e {
            fn eq(&self, other: &str) -> bool {
                self.as_ref().eq(other)
            }
        }

        impl ::std::str::FromStr for $e {
            type Err = DataError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::try_new(s.into())
            }
        }
    };
}

identifier_impl!(EntryType, EntryTypeRef, ArchivedEntryType);
identifier_impl!(FieldKey, FieldKeyRef, ArchivedFieldKey);
identifier_impl!(FieldValue, FieldValueRef, ArchivedFieldValue);

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

#[inline]
pub fn validate_ascii_identifier(s: &[u8]) -> Result<&str, DataError> {
    if s.is_empty() {
        return Err(DataError::Token(TokenError::Empty));
    }

    match s.iter().find(|&b| !ASCII_IDENTIFIER_ALLOWED[*b as usize]) {
        Some(b) => match char::try_from(*b) {
            Ok(ch) => Err(DataError::Token(TokenError::InvalidChar(ch))),
            Err(_) => Err(DataError::NonAscii),
        },
        // SAFETY: the only bytes permitted by ASCII_IDENTIFIER_ALLOWED are valid ASCII
        None => Ok(unsafe { std::str::from_utf8_unchecked(s) }),
    }
}
