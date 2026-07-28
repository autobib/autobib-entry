use std::{borrow::Borrow, collections::BTreeMap, str::FromStr, sync::LazyLock};

use serde::{Deserialize, de::Error};

use regex::Regex;

use crate::{
    data::EntryData,
    error::DataError,
    ident::{
        EntryType, EntryTypeRef, FieldKey, FieldKeyRef, FieldValue, FieldValueRef,
        StandardEntryType, StandardFieldKey,
    },
    normalize::{Normalize, normalize_whitespace_str},
};

/// An [`EntryData`] implementation which supports performant addition and deletion of fields.
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct MutableEntryData {
    entry_type: EntryType,
    fields: BTreeMap<FieldKey, FieldValue>,
}

impl Default for MutableEntryData {
    fn default() -> Self {
        Self::new(EntryType::default())
    }
}

impl MutableEntryData {
    /// Initialize with a given entry type.
    pub fn new(entry_type: EntryType) -> Self {
        Self {
            entry_type,
            fields: BTreeMap::new(),
        }
    }

    /// Initialize with a given standard entry type.
    pub fn new_standard(entry_type: StandardEntryType) -> Self {
        Self {
            entry_type: entry_type.into(),
            fields: BTreeMap::new(),
        }
    }

    /// Try to initialize a new instance, failing if the provided entry type is not valid.
    pub fn try_new<E: Into<String>>(e: E) -> Result<Self, DataError> {
        Ok(Self::new(EntryType::new(e.into())?))
    }

    /// Construct this instance by copying in data from other entry data.
    pub fn from_entry_data<D: EntryData + ?Sized>(cont: &D) -> Self {
        let mut new = Self::new(cont.entry_type().into());
        for (key, value) in cont.fields() {
            new.fields.insert(key.into(), value.into());
        }
        new
    }

    /// Insert a new field key and filed value.
    #[inline]
    pub fn insert(&mut self, key: FieldKey, value: FieldValue) -> Option<FieldValue> {
        self.fields.insert(key, value)
    }

    /// Insert a new standard field key and field value.
    #[inline]
    pub fn insert_standard(
        &mut self,
        key: StandardFieldKey,
        value: FieldValue,
    ) -> Option<FieldValue> {
        self.fields.insert(key.into(), value)
    }

    /// Insert a new standard field key and field value as a string.
    #[inline]
    pub fn insert_standard_key<V: Into<String>>(
        &mut self,
        key: StandardFieldKey,
        value: V,
    ) -> Result<Option<FieldValue>, DataError> {
        let value = FieldValue::new(value.into())?;
        Ok(self.fields.insert(key.into(), value))
    }

    /// Try to insert a new field key and field value as a string.
    #[inline]
    pub fn try_insert<K: Into<String>, V: Into<String>>(
        &mut self,
        k: K,
        v: V,
    ) -> Result<Option<FieldValue>, DataError> {
        Ok(self.insert(FieldKey::new(k.into())?, FieldValue::new(v.into())?))
    }

    /// Get the value at the key.
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<FieldValueRef<'_>>
    where
        FieldKey: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.fields.get::<Q>(key).map(FieldValue::by_ref)
    }

    /// Get the value at the key as a string.
    pub fn get_str<Q>(&self, key: &Q) -> Option<&str>
    where
        FieldKey: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.fields.get::<Q>(key).map(AsRef::as_ref)
    }

    /// Remove the value at the given key
    #[inline]
    pub fn remove<Q>(&mut self, key: &Q) -> Option<FieldValue>
    where
        FieldKey: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.fields.remove::<Q>(key)
    }
}

impl EntryData for MutableEntryData {
    fn fields(&self) -> impl IntoIterator<Item = (FieldKeyRef<'_>, FieldValueRef<'_>)> {
        self.fields.iter().map(|(k, v)| (k.by_ref(), v.by_ref()))
    }

    fn entry_type(&self) -> EntryTypeRef<'_> {
        self.entry_type.by_ref()
    }

    fn get_field<'r>(&'r self, field_name: &str) -> Option<FieldValueRef<'r>> {
        self.fields.get(field_name).map(FieldValue::by_ref)
    }

    fn count_fields(&self) -> usize {
        self.fields.len()
    }

    fn contains_field(&self, field_name: &str) -> bool {
        self.fields.contains_key(field_name)
    }
}

static TRAILING_JOURNAL_SERIES_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*\([1-9][0-9]*\)$").unwrap());

impl Normalize for MutableEntryData {
    fn set_eprint<Q: AsRef<str>>(&mut self, keys: std::slice::Iter<'_, Q>) -> bool {
        for key in keys {
            match self.is_eprint_normalized(key) {
                EPrintState::Ok => {
                    return false;
                }
                EPrintState::NeedsUpdate(val) => {
                    self.insert(FieldKey("eprint".into()), val.into());
                    // SAFETY: 'eprinttype' satisfies the key requirements
                    // SAFETY: `key` is already a key in the database, and the requirements for
                    // keys are stricter than the requirements for values.
                    self.insert(
                        FieldKey("eprinttype".into()),
                        FieldValue(key.as_ref().to_owned()),
                    );
                    return true;
                }
                EPrintState::MissingKey => {}
            }
        }
        false
    }

    fn normalize_whitespace(&mut self) -> bool {
        let mut updated = false;

        for val in self.fields.values_mut() {
            if let Some(new_val) = normalize_whitespace_str(val.0.as_ref()) {
                updated = true;
                // SAFETY: the `normalize_whitespace` function always reduces the length of the
                // input, since it either deletes unused whitespace, or replaces whitespace
                // with ASCII space which has the smallest possible length (as bytes)
                *val = FieldValue(new_val);
            }
        }

        updated
    }

    fn strip_journal_series(&mut self) -> bool {
        if let Some(journal) = self.fields.get_mut("journal")
            && let Some(truncate_offset) = TRAILING_JOURNAL_SERIES_RE
                .find(journal.0.as_ref())
                .map(|m| m.start())
        {
            // SAFETY: the new value is a prefix of the previous value, and the regex
            // guarantees that it will not result in unbalanced {}
            journal.0.truncate(truncate_offset);
            return true;
        }
        false
    }
}

/// The result of checking the current state of the `eprint` and `eprinttype` relative to a provided
/// key.
enum EPrintState<S> {
    /// No changes required.
    Ok,
    /// The `eprint` field corresponding to the provided key needs to be updated with the provided
    /// value.
    NeedsUpdate(S),
    /// The given key was not present in the record.
    MissingKey,
}

#[derive(Debug, Clone)]
pub struct SetFieldCommand {
    pub field_key: FieldKey,
    pub field_value: FieldValue,
}

impl FromStr for SetFieldCommand {
    type Err = serde_bibtex::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut reader = serde_bibtex::StrReader::new(s);
        let key = reader.read_field_key()?;
        reader.skip_field_sep()?;
        let value = reader.read_text_token()?;
        let field_key = FieldKey::from_str(key.into_inner()).map_err(Self::Err::custom)?;
        let field_value = FieldValue::from_str(value)
            .map_err(Self::Err::custom)?
            .to_owned();
        Ok(Self {
            field_key,
            field_value,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct EntryEditCommand {
    pub update_entry_type: Option<EntryType>,
    pub delete_field: Vec<FieldKey>,
    pub set_field: Vec<SetFieldCommand>,
}

impl EntryEditCommand {
    pub fn is_identity(&self) -> bool {
        self.set_field.is_empty()
            && self.delete_field.is_empty()
            && self.update_entry_type.is_none()
    }
}

/// The outcome of resolving the conflict when using [`MutableEntryData::merge_with_callback`].
pub enum ConflictResolved<T = FieldValue> {
    /// Keep the current data.
    Current,
    /// Replace with incoming data.
    Incoming,
    /// Use new data.
    New(T),
}

impl MutableEntryData {
    pub fn edit(&mut self, cmd: &EntryEditCommand) -> bool {
        let mut changed = false;

        if let Some(ref ty) = cmd.update_entry_type {
            changed |= self.update_entry_type(ty);
        }

        for key in &cmd.delete_field {
            changed |= self.remove(key).is_some();
        }

        for cmd in &cmd.set_field {
            changed |= self.set_field(cmd);
        }

        changed
    }

    /// Update the entry type to have the new value, returning if the entry type changed.
    pub fn update_entry_type(&mut self, ty: &EntryType) -> bool {
        if &self.entry_type != ty {
            self.entry_type = ty.clone();
            true
        } else {
            false
        }
    }

    /// Set a field using the provided command.
    pub fn set_field(&mut self, cmd: &SetFieldCommand) -> bool {
        let mut changed = false;
        self.fields
            .entry(cmd.field_key.clone())
            .and_modify(|v| {
                if v != &cmd.field_value {
                    *v = cmd.field_value.clone();
                    changed = true;
                }
            })
            .or_insert_with(|| {
                changed = true;
                cmd.field_value.clone()
            });
        changed
    }

    /// Check for the following configuration inside the data:
    /// ```bib
    ///   eprinttype = {key},
    ///   eprint = {val},
    ///   key = {val},
    /// ```
    /// If the key is missing, returns `EPrintState::Missing`; otherwise, check if the `eprinttype`
    /// and `eprint` keys require changing.
    fn is_eprint_normalized<Q: AsRef<str>>(&self, key: Q) -> EPrintState<FieldValueRef<'_>> {
        let key_ref = key.as_ref();
        match self.get(key_ref) {
            Some(val) => {
                if self.get("eprinttype").is_some_and(|v| v.0 == key_ref)
                    && self.get("eprint").is_some_and(|v| v.0 == val.0)
                {
                    EPrintState::Ok
                } else {
                    EPrintState::NeedsUpdate(val)
                }
            }
            None => EPrintState::MissingKey,
        }
    }

    /// This method is very similar to `merge_or_overwrite`, but also updates the entry type and is
    /// slightly more optimized since it blindly overwrites existing entries, instead of checking
    /// that they are different.
    pub fn update_from<D: EntryData + ?Sized>(&mut self, data: &D) {
        self.entry_type.0.clear();
        self.entry_type.0.push_str(data.entry_type().inner());

        for (key, value) in data.fields() {
            match self.fields.get_mut(key.inner()) {
                Some(existing) => {
                    existing.0.clear();
                    existing.0.push_str(value.inner());
                }
                None => {
                    self.fields.insert(key.into(), value.into());
                }
            }
        }
    }

    /// Merge data from `other`, invoking a callback to resolve conflicts.
    ///
    /// The callback `resolve_conflict` takes three arguments in the following order:
    /// the key, the existing value in `self` corresponding to the key, and the new value.
    pub fn merge_with_callback<
        D: EntryData + ?Sized,
        T: FnOnce(EntryTypeRef<'_>, EntryTypeRef<'_>) -> ConflictResolved<EntryType>,
        F: FnMut(FieldKeyRef<'_>, FieldValueRef<'_>, FieldValueRef<'_>) -> ConflictResolved,
    >(
        &mut self,
        other: &D,
        resolve_entry_type_conflict: T,
        mut resolve_field_conflict: F,
    ) {
        let other_entry_type = other.entry_type();
        if self.entry_type != other_entry_type {
            match resolve_entry_type_conflict(self.entry_type.by_ref(), other_entry_type) {
                ConflictResolved::Current => {}
                ConflictResolved::Incoming => {
                    self.entry_type = other_entry_type.into();
                }
                ConflictResolved::New(value) => {
                    self.entry_type = value;
                }
            }
        }

        for (key, value) in other.fields() {
            match self.fields.get_mut(key.inner()) {
                Some(current_value) if current_value != &value => {
                    match resolve_field_conflict(key, current_value.by_ref(), value) {
                        ConflictResolved::Current => continue,
                        ConflictResolved::Incoming => {
                            current_value.0.clear();
                            current_value.0.push_str(value.inner());
                        }
                        ConflictResolved::New(new_value) => {
                            *current_value = new_value;
                        }
                    };
                }
                Some(_) => {}
                None => {
                    self.fields.insert(key.into(), value.into());
                }
            }
        }
    }

    /// Merge data from `other`, ignoring fields that already exist in `self`.
    #[inline]
    pub fn merge_or_skip<D: EntryData + ?Sized>(&mut self, other: &D) {
        self.merge_with_callback(
            other,
            |_, _| ConflictResolved::Current,
            |_, _, _| ConflictResolved::Current,
        );
    }

    /// Merge data from `other`, overwriting fields that already exist in `self`.
    #[inline]
    pub fn merge_or_overwrite<D: EntryData + ?Sized>(&mut self, other: &D) {
        self.merge_with_callback(
            other,
            |_, _| ConflictResolved::Incoming,
            |_, _, _| ConflictResolved::Incoming,
        );
    }
}
