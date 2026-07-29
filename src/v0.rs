//! # `v0` zero-copy deserialization format
//!
//! This is the legacy `v0` data format originally used by Autobib versions `< 0.7.0`.
//!
//! ## Format
//!
//! The data is stored as a sequence of blocks.
//! ```txt
//! HEADER, TYPE, DATA1, DATA2, ...
//! ```
//! The `HEADER` is a single byte which is `0` and the `TYPE` consists of
//! ```txt
//! [entry_type_len: u8, entry_type: [u8..]]
//! ```
//! Here, `entry_type_len` is the length of `entry_type`, which has length at most [`u8::MAX`].
//! Then, each block `DATA` is of the form
//! ```txt
//! [key_len: u8, value_len: u16, key: [u8..], value: [u8..]]
//! ```
//! where `key_len` is the length of the first `key` segment, and the `value_len` is
//! the length of the `value` segment. Necessarily, `key` and `value` have lengths at
//! most [`u8::MAX`] and [`u16::MAX`] respectively.
//!
//! `value_len` is encoded in little endian format.
//!
//! The `DATA...` are sorted by `key` and each `key` and `entry_type` must be ASCII lowercase. The
//! `entry_type` can be any valid UTF-8.
use std::str::{from_utf8, from_utf8_unchecked};

use crate::error::AccessError;
use crate::{
    data::{Archive, EntryData},
    ident::{EntryTypeRef, FieldKeyRef, FieldValueRef},
};

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
/// of fields, see [`MutableEntryData`](crate::data::MutableEntryData).
#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct ArchivedEntryData([u8]);

impl ToOwned for ArchivedEntryData {
    type Owned = Box<Self>;

    fn to_owned(&self) -> Self::Owned {
        unsafe { Self::load_unchecked(Box::from(&self.0)) }
    }
}

unsafe impl Archive for ArchivedEntryData {
    /// Obtain the underlying bytes.
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Check that raw bytes are in the memory format defined in the module level documentation.
    fn validate(bytes: &[u8]) -> Result<(), AccessError> {
        match bytes {
            [0, ..] => {
                let mut cursor = Self::check_type(bytes, 1)?;
                let mut idx = 0;
                loop {
                    match Self::check_data_block(bytes, idx, cursor)? {
                        Some(next_cursor) => {
                            idx += 1;
                            cursor = next_cursor;
                        }
                        None => break Ok(()),
                    }
                }
            }
            [_, ..] => Err(AccessError::Unrecognized),
            [] => Err(AccessError::InvalidHeader),
        }
    }

    /// # Safety
    ///
    /// The buffer bytes must be in the format as specified in the module-level documentation. This
    /// is guaranteed if [`Self::validate`] returns Ok, or if the buffer was originally produced by
    /// [`archive`] or [`Self::as_bytes`].
    unsafe fn load_unchecked(buf: Box<[u8]>) -> Box<Self> {
        unsafe { Box::from_raw(Box::into_raw(buf) as *mut ArchivedEntryData) }
    }

    /// # Safety
    ///
    /// The buffer bytes must be in the format as specified in the module-level documentation. This
    /// is guaranteed if [`Self::validate`] returns Ok, or if the buffer was originally produced by
    /// [`archive`] or [`Self::as_bytes`].
    unsafe fn access_unchecked(b: &[u8]) -> &Self {
        unsafe { std::mem::transmute(b) }
    }

    /// Construct the byte representation from any entry data implementation.
    fn from_entry_data<D: EntryData>(data: D) -> Box<ArchivedEntryData> {
        let entry_data: &D = &data;
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

        unsafe { ArchivedEntryData::load_unchecked(data.into_boxed_slice()) }
    }

    /// Convert into boxed bytes.
    #[inline]
    fn into_archive(archive: Box<Self>) -> Box<[u8]> {
        unsafe { Box::from_raw(Box::into_raw(archive) as *mut [u8]) }
    }
}

impl ArchivedEntryData {
    /// Check that the `entry type` block is valid and return the updated cursor position.
    fn check_type(data: &[u8], cursor: usize) -> Result<usize, AccessError> {
        match data[cursor..] {
            [entry_type_len, ..] => {
                let entry_type_start = cursor + 1;
                let entry_type_end = entry_type_start + entry_type_len as usize;
                let entry_type_bytes = data
                    .get(entry_type_start..entry_type_end)
                    .ok_or(AccessError::InvalidEntryType)?;

                from_utf8(entry_type_bytes)?;

                Ok(entry_type_end)
            }
            _ => Err(AccessError::InvalidEntryType),
        }
    }

    /// Check that a `data block` is valid. If there are no more blocks, return `Ok(None)`;
    /// otherwise, return the updated cursor position.
    fn check_data_block(
        data: &[u8],
        idx: usize,
        cursor: usize,
    ) -> Result<Option<usize>, AccessError> {
        match data[cursor..] {
            [key_len, value_len_0, value_len_1, ..] => {
                let value_len = u16::from_le_bytes([value_len_0, value_len_1]) as usize;

                let key_block_start = cursor + 3;
                let value_block_start = key_block_start + key_len as usize;
                let value_block_end = value_block_start + value_len;

                let key_bytes = data
                    .get(key_block_start..value_block_start)
                    .ok_or(AccessError::InvalidIndex(idx))?;
                let value_bytes = data
                    .get(value_block_start..value_block_end)
                    .ok_or(AccessError::InvalidIndex(idx))?;

                from_utf8(key_bytes)?;
                from_utf8(value_bytes)?;

                Ok(Some(value_block_end))
            }
            [] => Ok(None),
            _ => Err(AccessError::IncompleteFields),
        }
    }

    /// Split into the `TYPE` and `DATA` blocks, discarding the header.
    #[inline]
    fn split_blocks(&self) -> (&[u8], &[u8]) {
        unsafe {
            let contents = self.0.get_unchecked(DATA_HEADER_SIZE..);
            contents.split_at_unchecked(*contents.get_unchecked(0) as usize + 1)
        }
    }
}

/// The iterator type for the fields of a [`LegacyEntryData`]. This cannot be constructed directly;
/// it is constructed implicitly by the [`EntryData::fields`] implementation of [`LegacyEntryData`].
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
            let key_len = unsafe { *self.remaining.get_unchecked(0) as usize };
            let value_len = unsafe {
                u16::from_le_bytes(
                    *self
                        .remaining
                        .get_unchecked(1..3)
                        .as_array::<2>()
                        .unwrap_unchecked(),
                ) as usize
            };
            let tail = unsafe { self.remaining.get_unchecked(3..) };

            let (key, tail) = unsafe { tail.split_at_unchecked(key_len) };
            let (value, tail) = unsafe { tail.split_at_unchecked(value_len) };

            self.remaining = tail;

            unsafe {
                Some((
                    FieldKeyRef(from_utf8_unchecked(key)),
                    FieldValueRef(from_utf8_unchecked(value)),
                ))
            }
        } else {
            None
        }
    }
}

impl EntryData for ArchivedEntryData {
    fn fields(&self) -> impl IntoIterator<Item = (FieldKeyRef<'_>, FieldValueRef<'_>)> {
        let (_, data) = self.split_blocks();
        LegacyFieldsIter { remaining: data }
    }

    fn entry_type(&self) -> EntryTypeRef<'_> {
        let (type_block, _) = self.split_blocks();
        unsafe { EntryTypeRef(from_utf8_unchecked(type_block.get_unchecked(1..))) }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data::MutableEntryData;

    /// Check that conversion into the raw form and back results in identical data.
    #[test]
    fn test_data_round_trip() {
        let mut record_data = MutableEntryData::try_new("article").unwrap();
        record_data.try_insert("year", "2024").unwrap();
        record_data.try_insert("title", "A title").unwrap();
        record_data.try_insert("field", "").unwrap();
        record_data.try_insert("a".repeat(255), "🍄").unwrap();
        record_data.try_insert("a", "b".repeat(65_535)).unwrap();

        let raw_data = ArchivedEntryData::from_entry_data(&record_data);

        let mut record_data_clone =
            MutableEntryData::try_new(raw_data.entry_type().inner()).unwrap();

        for (key, value) in raw_data.fields() {
            record_data_clone
                .try_insert(key.inner(), value.inner())
                .unwrap();
        }

        assert_eq!(record_data, record_data_clone);
        assert_eq!(
            raw_data.as_bytes(),
            ArchivedEntryData::from_entry_data(&record_data_clone).as_bytes()
        );
    }

    #[test]
    fn test_round_trip() {
        fn check(keys: &[(&'static str, &'static str)]) {
            let mut data = MutableEntryData::default();
            for (k, v) in keys {
                data.try_insert(*k, *v).unwrap();
            }
            assert_eq!(data.fields().into_iter().count(), keys.len());

            let raw_data = ArchivedEntryData::from_entry_data(&data);
            assert_eq!(raw_data.fields().into_iter().count(), keys.len());

            let new_data = MutableEntryData::from_entry_data(raw_data.as_ref());
            assert_eq!(new_data.fields().into_iter().count(), keys.len());

            for (k, v) in keys {
                assert_eq!(raw_data.get_field_str(k), Some(*v));
                assert_eq!(data.get_field_str(k), Some(*v));
                assert_eq!(new_data.get_field_str(k), Some(*v));
            }
        }
        check(&[("a", "A"), ("b", "B")]);
        check(&[("a", "A"), ("c", ""), ("b", "C")]);
        check(&[]);
        check(&[("b", "a")]);
    }

    #[test]
    fn test_format_manual() {
        let mut record_data = MutableEntryData::try_new("article").unwrap();
        record_data.try_insert("year", "2023").unwrap();
        record_data.try_insert("title", "The Title").unwrap();

        let data = ArchivedEntryData::from_entry_data(&record_data);
        let expected = vec![
            0, 7, b'a', b'r', b't', b'i', b'c', b'l', b'e', 5, 9, 0, b't', b'i', b't', b'l', b'e',
            b'T', b'h', b'e', b' ', b'T', b'i', b't', b'l', b'e', 4, 4, 0, b'y', b'e', b'a', b'r',
            b'2', b'0', b'2', b'3',
        ];

        assert_eq!(expected, data.as_bytes());
    }

    #[test]
    fn test_validate_data_ok() {
        for data in [
            // usual example
            vec![
                0, 7, b'a', b'r', b't', b'i', b'c', b'l', b'e', 5, 9, 0, b't', b'i', b't', b'l',
                b'e', b'T', b'h', b'e', b' ', b'T', b'i', b't', b'l', b'e', 4, 4, 0, b'y', b'e',
                b'a', b'r', b'2', b'0', b'2', b'3',
            ],
            // no keys is OK
            vec![0, 7, b'a', b'r', b't', b'i', b'c', b'l', b'e'],
            // field value can have length 0
            vec![0, 1, b'a', 1, 0, 0, b'b'],
            // usual example
            vec![
                0, 7, b'a', b'r', b't', b'i', b'c', b'l', b'e', 5, 9, 0, b't', b'i', b't', b'l',
                b'e', b'T', b'h', b'e', b' ', b'T', b'i', b't', b'l', b'e', 4, 4, 0, b'y', b'e',
                b'a', b'r', b'2', b'0', b'2', b'3',
            ],
        ] {
            assert!(ArchivedEntryData::access(&data).is_ok());
        }
    }

    #[test]
    fn test_validate_data_err() {
        // invalid version
        let malformed_data = vec![
            2, 7, b'a', b'r', b't', b'i', b'c', b'l', b'e', 5, 9, 0, b't', b'i', b't', b'l', b'e',
            b'T', b'h', b'e', b' ', b'T', b'i', b't', b'l', b'e', 4, 4, 0, b'y', b'e', b'a', b'r',
            b'2', b'0', b'2', b'3',
        ];
        let parsed = ArchivedEntryData::access(&malformed_data);
        assert_eq!(parsed, Err(AccessError::Unrecognized));

        // entry type is not valid utf-8
        let malformed_data = vec![
            0, 7, b'a', b'r', b't', 255, b'c', b'l', b'e', 5, 9, 0, b't', b'i', b't', b'l', b'e',
            b'T', b'h', b'e', b' ', b'T', b'i', b't', b'l', b'e', 4, 4, 0, b'y', b'e', b'a', b'r',
            b'2', b'0', b'2', b'3',
        ];
        let parsed = ArchivedEntryData::access(&malformed_data);
        std::assert_matches!(parsed, Err(AccessError::InvalidUtf8(_)));

        // bad length header
        let malformed_data = vec![
            0, 7, b'a', b'r', b't', b'i', b'c', b'l', b'e', 5, 100, 0, b't', b'i', b't', b'l',
            b'e', b'T', b'h', b'e', b' ', b'T', b'i', b't', b'l', b'e', 4, 4, 0, b'y', b'e', b'a',
            b'r', b'2', b'0', b'2', b'3',
        ];
        let parsed = ArchivedEntryData::access(&malformed_data);
        assert_eq!(parsed, Err(AccessError::InvalidIndex(0)));

        // trailing bytes
        let malformed_data = vec![0, 7, b'a', b'r', b't', b'i', b'c', b'l', b'e', 1];
        let parsed = ArchivedEntryData::access(&malformed_data);
        assert!(parsed.is_err());
    }
}
