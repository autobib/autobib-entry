//! # Entry data types
//!
//! This module contains the core abstractions over entry data.
//!
//! - [`EntryData`]: a trait representing types which encapsulate the data content of a single BibTeX entry.
//! - [`MutableEntryData`]: an [`EntryData`] implementation which also permits performant mutation.
//!   Used to construct typed entry data.
//! - [`ArchivedEntryData`]: a zero-copy deserialization format used for accessing archived data and
//!   for archiving entry data.
//! - [`archive`]: a convenience function to convert an [`EntryData`] directly into the archived
//!   archived format.
//! - [`EntryDataSerializer`]: a wrapper around an [`EntryData`] implementation which implements
//!   [`serde::Serialize`] to allow serialization into other serde-compatible formats.
mod mutable;
mod normalize;

use std::ops::Deref;

use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeStruct},
};

use crate::{
    error::{AccessError, DataError},
    ident::{EntryType, EntryTypeRef, FieldKey, FieldKeyRef, FieldValue, FieldValueRef},
};

pub use self::{
    mutable::{ConflictResolved, EntryEditCommand, MutableEntryData, SetFieldCommand},
    normalize::{Normalization, Normalize},
};

/// This trait represents types which encapsulate the data content of a single BibTeX entry.
pub trait EntryData {
    /// Iterate over `(key, value)` pairs in order.
    fn fields(&self) -> impl IntoIterator<Item = (FieldKeyRef<'_>, FieldValueRef<'_>)>;

    /// Get the entry type.
    fn entry_type(&self) -> EntryTypeRef<'_>;

    /// Count the number of fields.
    ///
    /// The default implementation uses `self.fields().into_iter().count()`. Implementations
    /// should provide a more performative alternative when possible.
    fn count_fields(&self) -> usize {
        self.fields().into_iter().count()
    }

    /// Get the value of a given field.
    ///
    /// The default implementation iterates over all fields and returns the first match.
    fn get_field<'r>(&'r self, field_name: &str) -> Option<FieldValueRef<'r>> {
        for (key, val) in self.fields() {
            if field_name < key.inner() {
                return None;
            }

            if field_name == key.inner() {
                return Some(val);
            }
        }
        None
    }

    /// Get the value of the field as a string.
    fn get_field_str<'r>(&'r self, field_name: &str) -> Option<&'r str> {
        self.get_field(field_name).map(|k| k.inner())
    }

    /// Check if a given field exists.
    ///
    /// The default implementation checks that `get_field` returns `Some(_)`.
    fn contains_field(&self, field_name: &str) -> bool {
        self.get_field(field_name).is_some()
    }

    /// Validate that this entry data implementation is correct.
    fn validate_untrusted(&self) -> Result<(), DataError> {
        if !self.fields().into_iter().is_sorted_by_key(|(k, _)| k) {
            return Err(DataError::Unsorted);
        }

        EntryType::validate(self.entry_type().inner())?;

        for (k, v) in self.fields() {
            FieldKey::validate(k.inner())?;
            FieldValue::validate(v.inner())?;
        }

        Ok(())
    }
}

/// Serialize entry data as raw bytes in the given format.
pub fn archive<A: Archive + ?Sized, D: EntryData>(data: D) -> Box<[u8]> {
    A::into_archive(A::from_entry_data(data))
}

/// Types that can be converted to raw bytes, which can be deserialized from raw bytes, and for
/// which data can be immutably read from a byte slice.
pub unsafe trait Archive: ToOwned {
    /// Obtain the underlying bytes.
    fn as_bytes(&self) -> &[u8];

    /// Convert this type into an owned byte slice.
    fn into_archive(archive: Self::Owned) -> Box<[u8]>;

    /// Check that a byte slice is in a valid format accepted by this type.
    fn validate(bytes: &[u8]) -> Result<(), AccessError>;

    /// Construct a new instance of this type from any entry data.
    fn from_entry_data<D: EntryData>(data: D) -> Self::Owned;

    /// Load the provided byte buffer without checking that the underlying bytes are valid.
    ///
    /// # Safety
    ///
    /// If the underlying bytes are not valid according to the format specified in the module-level
    /// documentation, this is undefined behaviour. The format is guaranteed to be correct if:
    ///
    /// - [`Self::validate`] returns ok.
    /// - The bytes were produced by a call to [`archive`] or [`Self::as_bytes`].
    unsafe fn load_unchecked(bytes: Box<[u8]>) -> Self::Owned;

    /// Access data from the provided byte buffer without any copying or parsing, without
    /// checking that the underlying bytes are valid.
    ///
    /// # Safety
    ///
    /// If the underlying bytes are not valid according to the format specified in the module-level
    /// documentation, this is undefined behaviour. The format is guaranteed to be correct if:
    ///
    /// - [`Self::validate`] returns ok.
    /// - The bytes were produced by a call to [`archive`] or [`Self::as_bytes`].
    unsafe fn access_unchecked(bytes: &[u8]) -> &Self;

    /// Load the provided byte buffer, first checking that the underlying bytes are valid.
    ///
    /// The default implementation first [validates](Self::validate) the byte buffer and then calls
    /// [`load_unchecked`](Self::load_unchecked).
    fn load(bytes: Box<[u8]>) -> Result<Self::Owned, AccessError> {
        Self::validate(&bytes)?;
        unsafe { Ok(Self::load_unchecked(bytes)) }
    }

    /// Access data from the provided byte buffer without any copying or parsing, first
    /// checking that the underlying bytes are valid.
    ///
    /// The default implementation first [validates](Self::validate) the byte buffer and then calls
    /// [`access_unchecked`](Self::access_unchecked).
    fn access(bytes: &[u8]) -> Result<&Self, AccessError> {
        Self::validate(&bytes)?;
        unsafe { Ok(Self::access_unchecked(bytes)) }
    }
}

impl<D: Deref> EntryData for D
where
    D::Target: EntryData,
{
    #[inline]
    fn fields(&self) -> impl IntoIterator<Item = (FieldKeyRef<'_>, FieldValueRef<'_>)> {
        self.deref().fields()
    }

    #[inline]
    fn entry_type(&self) -> EntryTypeRef<'_> {
        self.deref().entry_type()
    }

    #[inline]
    fn count_fields(&self) -> usize {
        self.deref().count_fields()
    }

    #[inline]
    fn get_field<'r>(&'r self, field_name: &str) -> Option<FieldValueRef<'r>> {
        self.deref().get_field(field_name)
    }

    #[inline]
    fn get_field_str<'r>(&'r self, field_name: &str) -> Option<&'r str> {
        self.deref().get_field_str(field_name)
    }

    #[inline]
    fn contains_field(&self, field_name: &str) -> bool {
        self.deref().contains_field(field_name)
    }
}

/// A wrapper for an [`EntryData`] implementation which implements [`Serialize`].
pub struct EntryDataSerializer<'a, D: ?Sized> {
    data: &'a D,
}

impl<'a, D: EntryData + ?Sized> EntryDataSerializer<'a, D> {
    /// Wrap an entry data implementation.
    pub fn new(data: &'a D) -> Self {
        Self { data }
    }
}

impl<'a, D: EntryData + ?Sized> Serialize for EntryDataSerializer<'a, D> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        struct FieldsWrapper<'a, D: ?Sized>(&'a D);

        impl<'a, D: EntryData + ?Sized> Serialize for FieldsWrapper<'a, D> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut state = serializer.serialize_map(None)?;
                for (key, value) in self.0.fields() {
                    state.serialize_entry(&key, &value)?;
                }
                state.end()
            }
        }

        let mut state = serializer.serialize_struct("EntryData", 2)?;
        state.serialize_field("entry_type", &self.data.entry_type())?;
        state.serialize_field("fields", &FieldsWrapper(self.data))?;
        state.end()
    }
}
