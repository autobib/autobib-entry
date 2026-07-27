use std::str::from_utf8;

use crate::ident::validate_ascii_identifier;
use crate::{EntryData, EntryTypeRef, FieldKeyRef, FieldValueRef};

/// The size (in bytes) of the version header.
const DATA_HEADER_SIZE: usize = 1;

/// The type of integer used in the header for the BibTeX key.
pub(crate) type KeyHeader = u8;

/// The type of integer used in the header for the BibTeX value.
pub(crate) type ValueHeader = u16;

/// The type of integer used in the BibTeX entry type header.
pub(crate) type EntryTypeHeader = u8;

/// A raw binary representation of the field key and fields of a BibTeX entry.
///
/// This struct is immutable by design. For a mutable version which supports addition and deletion
/// of fields, see [`MutableEntryData`](super::MutableEntryData).
#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct LegacyEntryData([u8]);

pub fn serialize<D: EntryData + ?Sized>(entry_data: &D) -> Box<[u8]> {
    let raw_len = 1  // the size of the binary version header
            + (1 + entry_data.entry_type().inner().len()) // the entry type, plus the 1-byte header
            + entry_data // the key value pairs, plus the 3-byte header
                .fields()
                .into_iter()
                .map(|(k, v)| 3 + k.inner().len() + v.inner().len())
                .sum::<usize>();

    let mut data = Vec::with_capacity(raw_len);

    data.push(0);

    let entry_type = entry_data.entry_type();
    let entry_type_len = EntryTypeHeader::try_from(entry_type.inner().len()).unwrap();
    data.push(entry_type_len);
    data.extend(entry_type.inner().as_bytes());

    for (key, value) in entry_data.fields() {
        let key_len = KeyHeader::try_from(key.inner().len()).unwrap();
        let value_len = ValueHeader::try_from(value.inner().len())
            .unwrap()
            .to_le_bytes();

        data.push(key_len);
        data.extend(value_len);
        data.extend(key.inner().as_bytes());
        data.extend(value.inner().as_bytes());
    }

    data.into_boxed_slice()
}

impl LegacyEntryData {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn validate(bytes: &[u8]) -> Result<(), InvalidBytesError> {
        match bytes {
            [0, ..] => {
                let mut cursor = Self::check_type(bytes, 1)?;
                loop {
                    match Self::check_data_block(bytes, cursor)? {
                        Some(next_cursor) => {
                            cursor = next_cursor;
                        }
                        None => break Ok(()),
                    }
                }
            }
            [_, ..] => Err(InvalidBytesError::new(0, "invalid version")),
            [] => Err(InvalidBytesError::new(0, "data was empty")),
        }
    }

    pub fn load(bytes: Box<[u8]>) -> Result<Box<Self>, InvalidBytesError> {
        Self::validate(&bytes)?;
        unsafe { Ok(Self::load_unchecked(bytes)) }
    }

    /// # Safety
    ///
    /// The buffer bytes must be in the format as specified in the module-level documentation. This
    /// is guaranteed if [`Self::validate`] returns Ok, or if the buffer was originally produced by
    /// [`serialize`] or [`Self::as_bytes`].
    pub unsafe fn load_unchecked(buf: Box<[u8]>) -> Box<Self> {
        unsafe { Box::from_raw(Box::into_raw(buf) as *mut LegacyEntryData) }
    }

    pub fn access(bytes: &[u8]) -> Result<&Self, InvalidBytesError> {
        Self::validate(bytes)?;
        unsafe { Ok(Self::access_unchecked(bytes)) }
    }

    /// # Safety
    ///
    /// The buffer bytes must be in the format as specified in the module-level documentation. This
    /// is guaranteed if [`Self::validate`] returns Ok, or if the buffer was originally produced by
    /// [`serialize`] or [`Self::as_bytes`].
    pub unsafe fn access_unchecked(b: &[u8]) -> &Self {
        unsafe { std::mem::transmute(b) }
    }

    pub fn from_entry_data<D: EntryData + ?Sized>(data: &D) -> Box<LegacyEntryData> {
        unsafe { LegacyEntryData::load_unchecked(serialize(data)) }
    }
}

impl LegacyEntryData {
    /// Check that the `entry type` block is valid and return the updated cursor position.
    fn check_type(data: &[u8], cursor: usize) -> Result<usize, InvalidBytesError> {
        match data[cursor..] {
            [0, ..] => Err(InvalidBytesError::new(
                cursor,
                "entry type cannot have length zero",
            )),
            [entry_type_len, ..] => {
                let entry_type_start = cursor + 1;
                let entry_type_end = entry_type_start + entry_type_len as usize;
                let entry_type_bytes =
                    data.get(entry_type_start..entry_type_end)
                        .ok_or(InvalidBytesError::new(
                            entry_type_start,
                            "entry type shorter than header",
                        ))?;

                if validate_ascii_identifier(entry_type_bytes).is_err() {
                    return Err(InvalidBytesError::new(
                        entry_type_start,
                        "entry type contains non-ASCII chararacters or invalid ASCII characters",
                    ));
                }

                Ok(entry_type_end)
            }
            _ => Err(InvalidBytesError::new(cursor, "missing entry type")),
        }
    }

    /// Check that a `data block` is valid. If there are no more blocks, return `Ok(None)`;
    /// otherwise, return the updated cursor position.
    fn check_data_block(data: &[u8], cursor: usize) -> Result<Option<usize>, InvalidBytesError> {
        match data[cursor..] {
            [0, _, _, ..] => Err(InvalidBytesError::new(
                cursor,
                "key cannot have length zero",
            )),
            [key_len, value_len_0, value_len_1, ..] => {
                let value_len = u16::from_le_bytes([value_len_0, value_len_1]) as usize;

                let key_block_start = cursor + 3;
                let value_block_start = key_block_start + key_len as usize;
                let value_block_end = value_block_start + value_len;

                let key_bytes =
                    data.get(key_block_start..value_block_start)
                        .ok_or(InvalidBytesError::new(
                            key_block_start,
                            "key block shorter than header",
                        ))?;
                let value_bytes =
                    data.get(value_block_start..value_block_end)
                        .ok_or(InvalidBytesError::new(
                            value_block_start,
                            "value block shorter than header",
                        ))?;

                if !serde_bibtex::token::is_balanced(value_bytes) {
                    return Err(InvalidBytesError::new(
                        value_block_start,
                        "value has unbalanced `{}`",
                    ));
                }

                if validate_ascii_identifier(key_bytes).is_err() {
                    return Err(InvalidBytesError::new(
                        key_block_start,
                        "field key contains non-ASCII chararacters or invalid ASCII characters",
                    ));
                }

                let _value = from_utf8(value_bytes).map_err(|e| {
                    InvalidBytesError::new(
                        value_block_start + e.valid_up_to(),
                        "value block has invalid utf-8 starting at position",
                    )
                })?;

                Ok(Some(value_block_end))
            }
            [] => Ok(None),
            _ => Err(InvalidBytesError::new(
                cursor,
                "incomplete data block header",
            )),
        }
    }

    /// Split into the `TYPE` and `DATA` blocks, discarding the header.
    #[inline]
    fn split_blocks(&self) -> (&[u8], &[u8]) {
        let contents = &self.0[DATA_HEADER_SIZE..];
        contents.split_at(contents[0] as usize + 1)
    }
}

/// The iterator type for the fields of a [`RawEntryData`]. This cannot be constructed directly;
/// it is constructed implicitly by the [`EntryData::fields`] implementation of [`RawEntryData`].
#[derive(Debug, Clone)]
pub struct LegacyFieldsIter<'a> {
    remaining: &'a [u8],
}

impl<'a> Iterator for LegacyFieldsIter<'a> {
    type Item = (FieldKeyRef<'a>, FieldValueRef<'a>);

    /// Iterate over the underlying `(key, value)` blocks.
    ///
    /// # Panics
    /// Panics if the underlying data is malformed.
    fn next(&mut self) -> Option<Self::Item> {
        if !self.remaining.is_empty() {
            let key_len = self.remaining[0] as usize;
            let value_len = u16::from_le_bytes([self.remaining[1], self.remaining[2]]) as usize;
            let tail = &self.remaining[3..];

            let (key, tail) = tail.split_at(key_len);
            let (value, tail) = tail.split_at(value_len);

            self.remaining = tail;

            Some((
                FieldKeyRef(from_utf8(key).unwrap()),
                FieldValueRef(from_utf8(value).unwrap()),
            ))
        } else {
            None
        }
    }
}

impl LegacyEntryData {
    pub fn raw_fields(&self) -> LegacyFieldsIter<'_> {
        let (_, data_blocks) = self.split_blocks();
        LegacyFieldsIter {
            remaining: data_blocks,
        }
    }
}

impl EntryData for LegacyEntryData {
    fn fields(&self) -> impl IntoIterator<Item = (FieldKeyRef<'_>, FieldValueRef<'_>)> {
        let (_, data) = self.split_blocks();
        LegacyFieldsIter { remaining: data }
    }

    fn entry_type(&self) -> EntryTypeRef<'_> {
        let (type_block, _) = self.split_blocks();
        EntryTypeRef(from_utf8(&type_block[1..]).unwrap())
    }
}

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
#[error("Invalid bytes: error at position `{position}`: {message}")]
pub struct InvalidBytesError {
    pub position: usize,
    pub message: &'static str,
}

impl InvalidBytesError {
    pub fn new(position: usize, message: &'static str) -> Self {
        Self { position, message }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum RecordDataError {
    #[error("Identifier contains invalid character")]
    ContainsInvalidChar,

    #[error(
        "Key has invalid size {0}; must be at least 1 and at most {max}",
        max = KeyHeader::MAX
    )]
    KeyInvalidLength(usize),

    #[error(
        "Entry type has invalid size {0}; must be at least 1 and at most {max}",
        max = EntryTypeHeader::MAX
    )]
    EntryTypeInvalidLength(usize),

    #[error("Entry type must not be one of the reserved names: comment, preamble, string")]
    EntryTypeReserved,

    #[error("Value has invalid size {0}; must be at most {max}", max = ValueHeader::MAX)]
    ValueInvalidLength(usize),

    #[error("Value does not contain balanced `{{ }}` braces")]
    ValueNotBalanced,

    #[error("Invalid bytes: `{0}`")]
    InvalidBytes(#[from] InvalidBytesError),
}
