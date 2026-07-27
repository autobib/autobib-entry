mod data;
mod error;
mod ident;
mod mutable;
mod normalize;
mod raw;
pub mod v0;

pub use {
    data::{EntryData, EntryDataSerializer},
    error::{DataError, DeserializationError},
    ident::{EntryType, EntryTypeRef, FieldKey, FieldKeyRef, FieldValue, FieldValueRef},
    mutable::MutableEntryData,
    normalize::{Normalization, Normalize},
    raw::{RawEntryData, serialize},
};
