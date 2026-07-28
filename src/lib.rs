pub mod data;
pub mod error;
mod ident;
mod normalize;

#[cfg(feature = "v0")]
pub mod v0;

pub use {
    error::{DataError, AccessError},
    ident::{EntryType, EntryTypeRef, FieldKey, FieldKeyRef, FieldValue, FieldValueRef},
    normalize::{Normalization, Normalize},
};
