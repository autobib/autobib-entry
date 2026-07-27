use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeStruct},
};

use crate::ident::{EntryTypeRef, FieldKeyRef, FieldValueRef};

/// This trait represents types which encapsulate the data content of a single BibTeX entry.
pub trait EntryData: PartialEq {
    /// Iterate over `(key, value)` pairs in order.
    fn fields(&self) -> impl IntoIterator<Item = (FieldKeyRef<'_>, FieldValueRef<'_>)>;

    /// Get the `entry_type` as a string slice.
    fn entry_type(&self) -> EntryTypeRef<'_>;

    /// The number of fields.
    ///
    /// The default implementation uses `self.fields().into_iter().count()`
    fn count_fields(&self) -> usize {
        self.fields().into_iter().count()
    }

    /// Get the value of the field.
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

    /// Check if the field exists.
    ///
    /// The default implementation checks that `get_field` returns `Some(_)`.
    fn contains_field(&self, field_name: &str) -> bool {
        self.get_field(field_name).is_some()
    }
}

/// A wrapper for an [`EntryData`] implementation which implements [`Serialize`].
pub struct EntryDataSerializer<'a, D: ?Sized> {
    data: &'a D,
}

impl<'a, D: EntryData + ?Sized> EntryDataSerializer<'a, D> {
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
