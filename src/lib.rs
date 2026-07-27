mod data;
mod error;
mod ident;
mod mutable;
mod normalize;
mod raw;

pub use {
    data::{EntryData, EntryDataSerializer},
    error::DataError,
    ident::{EntryType, EntryTypeRef, FieldKey, FieldKeyRef, FieldValue, FieldValueRef},
    mutable::MutableEntryData,
    normalize::{Normalization, Normalize},
    raw::RawEntryData,
};
