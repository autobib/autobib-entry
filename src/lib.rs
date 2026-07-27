pub mod data;
pub mod error;
mod ident;
mod mutable;
mod normalize;
mod raw;

pub use {
    ident::{EntryType, EntryTypeRef, FieldKey, FieldKeyRef, FieldValue, FieldValueRef},
    mutable::MutableEntryData,
    normalize::{Normalization, Normalize},
};
