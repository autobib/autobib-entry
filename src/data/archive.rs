//! # Zero-copy deserialization format
//!
//! ## Memory format
//!
//! All `u32` and `u64` values are stored in little-endian order.
//! ```text
//! | <- HEADER                       -> | <- FIELDS                          -> | <- DATA                 -> |
//! | meta | entry_type_len | num_fields | (key_idx, key_len, val_idx, val_len)* | entry_type.. keys.. vals.. |
//! | u64  | u32            | u32        | [u32, u32, u32, u32]*                 | str                        |
//! ```
//!
//! ### Format explanation
//!
//! - `HEADER`: fixed-size metadata for the data
//!   - `meta`: a currently unused metadata block, currently set as little-endian bytes to `[1 0 0 0 0 0 0 0]`.
//!     This distinguishes from the old data format used by Autobib which sets the first byte equal to `0`.
//!     For validity, only the first byte is checked.
//!     Future versions of this binary format may store additional metadata in the `meta` block.
//!   - `entry_type_len`: the length (in bytes) of the entry type
//!   - `num_fields`: the number of `key = {value}` fields
//! - `FIELDS`: variable-size metadata for each `key = {value}` field
//!   - `key_idx`: an index into ths byte buffer indicating the start of the `key`
//!   - `key_len`: the length of the `key`
//!   - `val_idx`: an index into ths byte buffer indicating the start of the `value`
//!   - `val_len`: the length of the `value`
//! - `DATA`: a contiguous string storing the raw contents of the entry type, and the field keys and the values.
//!   The indices in `FIELDS` refer to valid sub-strings of the `DATA` block.
//!
//! ### Format features
//!
//! - The fields are sorted by key.
//!   This means that specific `key = {value}` pairs can be found efficiently using [`binary_search_by_key`](https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search_by_key).
//! - The `DATA` block is a continguous Utf-8 string when valid.
//!   This improves initial validation since we can check Utf-8 validity in a single pass, rather than check validity for each key and value individually (2-3x slower in benchmarks).

use std::{ops::Range, str::from_utf8_unchecked};

const HEADER_LEN: usize = 16;
const FIELD_LEN: usize = 16;

use crate::{FieldKeyRef, FieldValueRef, data::EntryData, error::AccessError};

/// Archive the provided entry data as a raw byte buffer.
///
/// See [`ArchivedEntryData::from_entry_data`] for the typed variant of this function.
pub fn archive<D: EntryData + ?Sized>(data: &D) -> Box<[u8]> {
    let entry_type_bytes = data.entry_type().inner().as_bytes();
    let num_fields = data.count_fields();

    // pre-compute how much space we need since we will do non-linear allocation
    let header_required = HEADER_LEN;
    let fields_required = FIELD_LEN * num_fields;
    let data_start = header_required + fields_required;
    let str_total_len = data.entry_type().inner().len()
        + data
            .fields()
            .into_iter()
            .map(|(k, v)| k.inner().len() + v.inner().len())
            .sum::<usize>();

    let raw_data_len = data_start + str_total_len;
    assert!(
        raw_data_len < u32::MAX as usize,
        "Cannot write raw data exceeding 2^32 bytes!"
    );

    // initialize as zeroed; we will write non-sequentially
    let mut buf: Box<[u8]> = vec![0; raw_data_len].into_boxed_slice();

    // HEADER
    buf[0] = 1; // recall other values are zeroed
    buf[8..12].copy_from_slice(&(entry_type_bytes.len() as u32).to_le_bytes());
    buf[12..16].copy_from_slice(&(data.count_fields() as u32).to_le_bytes());

    // first, the entry data
    let mut offset = data_start;
    buf[offset..offset + entry_type_bytes.len()].copy_from_slice(entry_type_bytes);
    offset = data_start + entry_type_bytes.len();

    // then all of the fields
    for (idx, (k, v)) in data.fields().into_iter().enumerate() {
        let field_start = HEADER_LEN + FIELD_LEN * idx;

        // write the field key data and the field key
        buf[field_start..field_start + 4].copy_from_slice(&(offset as u32).to_le_bytes());
        buf[field_start + 4..field_start + 8]
            .copy_from_slice(&(k.inner().len() as u32).to_le_bytes());
        buf[offset..offset + k.inner().len()].copy_from_slice(k.inner().as_bytes());
        offset += k.inner().len();

        // write the field value data and the field value
        buf[field_start + 8..field_start + 12].copy_from_slice(&(offset as u32).to_le_bytes());
        buf[field_start + 12..field_start + 16]
            .copy_from_slice(&(v.inner().len() as u32).to_le_bytes());
        buf[offset..offset + v.inner().len()].copy_from_slice(v.inner().as_bytes());
        offset += v.inner().len();
    }

    buf
}

/// Entry data formatted as a raw byte buffer.
///
/// This is a typed wrapper around a `[u8]` and is in particular unsized.
#[derive(PartialEq)]
#[repr(transparent)]
pub struct ArchivedEntryData([u8]);

impl ArchivedEntryData {
    /// Obtain the underlying bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Check that raw bytes are in the memory format defined in the module level documentation.
    pub fn validate(bytes: &[u8]) -> Result<(), AccessError> {
        // checking header
        let Some((&[1, _, _, _, _, _, _, _, e0, e1, e2, e3, l0, l1, l2, l3], _)) =
            bytes.split_first_chunk::<HEADER_LEN>()
        else {
            return Err(AccessError::InvalidHeader);
        };
        let entry_type_len = u32::from_le_bytes([e0, e1, e2, e3]) as usize;
        let num_fields = u32::from_le_bytes([l0, l1, l2, l3]) as usize;

        // checking that there is data
        let data_start = HEADER_LEN + FIELD_LEN * num_fields;
        let Some(data) = bytes.get(data_start..) else {
            return Err(AccessError::IncompleteFields);
        };

        // checking string data is valid utf8
        let data_str = std::str::from_utf8(data)?;

        // checking continguous indices
        let mut kv_data_start = data_start + entry_type_len;

        for idx in 0..num_fields {
            let offset = HEADER_LEN + FIELD_LEN * idx;
            // we already checked that these will return valid indices with the length check above
            let (&field_bytes, _) = unsafe {
                bytes
                    .get_unchecked(offset..)
                    .split_first_chunk::<FIELD_LEN>()
                    .unwrap_unchecked()
            };
            let (key_idx, key_len, val_idx, val_len) = FieldAccess(field_bytes).parts();

            // check that the indices are contiguous and correspond to valid char boundaries
            if kv_data_start != key_idx {
                return Err(AccessError::InvalidIndex(idx));
            }

            if kv_data_start + key_len != val_idx {
                return Err(AccessError::InvalidIndex(idx));
            }

            if !data_str.is_char_boundary(key_idx - data_start) {
                return Err(AccessError::InvalidStrOffset(idx));
            }

            if !data_str.is_char_boundary(val_idx - data_start) {
                return Err(AccessError::InvalidStrOffset(idx));
            }

            kv_data_start = kv_data_start + key_len + val_len;
        }

        // we should end at bytes
        if kv_data_start != bytes.len() {
            return Err(AccessError::TrailingBytes(kv_data_start));
        }

        Ok(())
    }

    /// Load the provided byte buffer, first checking that the underlying bytes are valid.
    pub fn load(bytes: Box<[u8]>) -> Result<Box<Self>, AccessError> {
        Self::validate(&bytes)?;
        unsafe { Ok(Self::load_unchecked(bytes)) }
    }

    /// Load the provided byte buffer without checking that the underlying bytes are valid.
    ///
    /// # Safety
    ///
    /// If the underlying bytes are not valid according to the format specified in the module-level
    /// documentation, this is undefined behaviour. The format is guaranteed to be correct if:
    ///
    /// - [`Self::validate`] returns ok.
    /// - The bytes were produced by a call to [`archive`] or [`Self::as_bytes`].
    pub unsafe fn load_unchecked(buf: Box<[u8]>) -> Box<Self> {
        unsafe { Box::from_raw(Box::into_raw(buf) as *mut ArchivedEntryData) }
    }

    /// Access data from the provided byte buffer without any copying or parsing, first
    /// checking that the underlying bytes are valid.
    pub fn access(bytes: &[u8]) -> Result<&Self, AccessError> {
        Self::validate(bytes)?;
        unsafe { Ok(Self::access_unchecked(bytes)) }
    }

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
    pub unsafe fn access_unchecked(b: &[u8]) -> &Self {
        unsafe { std::mem::transmute(b) }
    }

    /// Construct the byte representation from any entry data implementation.
    pub fn from_entry_data<D: EntryData + ?Sized>(data: &D) -> Box<ArchivedEntryData> {
        unsafe { ArchivedEntryData::load_unchecked(archive(data)) }
    }
}

/// Layout data for the byte buffer, as read from the header.
struct RawLayout {
    entry_type_len: usize,
    num_fields: usize,
    data_start: usize,
}

impl RawLayout {
    /// Get the range in the data buffer corresponding to the entry type.
    #[inline]
    fn entry_type_range(&self) -> Range<usize> {
        self.data_start..self.data_start + self.entry_type_len
    }

    /// Get the subslice corresponding to the field metadata.
    #[inline]
    fn all_fields_range(&self) -> Range<usize> {
        HEADER_LEN..self.data_start
    }
}

/// An accessor for a single field.
#[derive(Debug, Clone, Copy)]
struct FieldAccess([u8; FIELD_LEN]);

impl FieldAccess {
    /// Split the field into its constituent `usize` parts: `key_idx`, `key_len`, `val_id`, and
    /// `val_len`.
    #[inline]
    fn parts(self) -> (usize, usize, usize, usize) {
        let [
            ki0,
            ki1,
            ki2,
            ki3,
            kl0,
            kl1,
            kl2,
            kl3,
            vi0,
            vi1,
            vi2,
            vi3,
            vl0,
            vl1,
            vl2,
            vl3,
        ] = self.0;
        (
            u32::from_le_bytes([ki0, ki1, ki2, ki3]) as usize,
            u32::from_le_bytes([kl0, kl1, kl2, kl3]) as usize,
            u32::from_le_bytes([vi0, vi1, vi2, vi3]) as usize,
            u32::from_le_bytes([vl0, vl1, vl2, vl3]) as usize,
        )
    }

    /// Access the contents of this field in the data buffer.
    ///
    /// # Safety
    ///
    /// The data buffer must be valid for the field from which this struct was constructed.
    #[inline]
    unsafe fn access_in<'r>(self, data: &'r [u8]) -> (FieldKeyRef<'r>, FieldValueRef<'r>) {
        let (key_idx, key_len, val_idx, val_len) = self.parts();
        unsafe {
            let key_raw = data.get_unchecked(key_idx..key_idx + key_len);
            let val_raw = data.get_unchecked(val_idx..val_idx + val_len);
            (
                FieldKeyRef(from_utf8_unchecked(key_raw)),
                FieldValueRef(from_utf8_unchecked(val_raw)),
            )
        }
    }
}

impl ArchivedEntryData {
    /// Obtain the layout from the header.
    #[inline]
    fn layout(&self) -> RawLayout {
        let &[e0, e1, e2, e3, n0, n1, n2, n3] = unsafe {
            self.0
                .get_unchecked(8..16)
                .as_array::<8>()
                .unwrap_unchecked()
        };
        let entry_type_len = u32::from_le_bytes([e0, e1, e2, e3]) as usize;
        let num_fields = u32::from_le_bytes([n0, n1, n2, n3]) as usize;
        let data_start = HEADER_LEN + FIELD_LEN * num_fields;
        RawLayout {
            entry_type_len,
            num_fields,
            data_start,
        }
    }

    /// Obtain the field metadata as a slice of `Field`s.
    #[inline]
    fn raw_fields(&self) -> &[[u8; FIELD_LEN]] {
        let ly = self.layout();
        unsafe {
            self.0
                .get_unchecked(ly.all_fields_range())
                .as_chunks_unchecked()
        }
    }
}

impl EntryData for ArchivedEntryData {
    fn fields(&self) -> impl IntoIterator<Item = (FieldKeyRef<'_>, FieldValueRef<'_>)> {
        let rf = self.raw_fields();
        unsafe {
            rf.iter()
                .map(|chunk| FieldAccess(*chunk).access_in(&self.0))
        }
    }

    fn entry_type(&self) -> crate::EntryTypeRef<'_> {
        let ly = self.layout();
        unsafe {
            crate::EntryTypeRef(from_utf8_unchecked(
                self.0.get_unchecked(ly.entry_type_range()),
            ))
        }
    }

    fn count_fields(&self) -> usize {
        self.layout().num_fields
    }

    fn get_field<'r>(&'r self, field_name: &str) -> Option<FieldValueRef<'r>> {
        let ly = self.layout();
        if ly.num_fields <= 6 {
            self.fields()
                .into_iter()
                .find(|(k, _)| k.inner() == field_name)
                .map(|(_, v)| v)
        } else {
            let rf = self.raw_fields();
            unsafe {
                rf.binary_search_by_key(&field_name, |&chunk| {
                    FieldAccess(chunk).access_in(&self.0).0.inner()
                })
                .ok()
                .map(|idx| FieldAccess(*rf.get_unchecked(idx)).access_in(&self.0).1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::*;

    #[test]
    fn basic() {
        let mut data = MutableEntryData::default();
        let fields = [
            ("author", "Alex Rutar"),
            ("journal", "Journal of Great Papers"),
            ("month", "Dec"),
            ("title", "A wonderful title"),
            ("year", "2036"),
        ];

        for (k, v) in fields {
            data.check_and_insert(k, v).unwrap();
        }

        let archived = ArchivedEntryData::from_entry_data(&data);
        assert_eq!(archived.count_fields(), fields.len());
        for (k, v) in fields {
            assert_eq!(archived.get_field_str(k), Some(v));
        }
        for ((k, v), (ser_k, ser_v)) in fields.iter().zip(archived.fields()) {
            assert_eq!(k, &ser_k.inner());
            assert_eq!(v, &ser_v.inner());
        }
    }

    #[test]
    fn round_trip() {
        let mut record_data = MutableEntryData::try_new("article").unwrap();
        let fields = [
            ("year", "2024"),
            ("title", "A title"),
            ("field", ""),
            ("weird", "🍄"),
            (&"a".repeat(255), &"b".repeat(65_535)),
        ];

        for (k, v) in fields {
            record_data.check_and_insert(k, v).unwrap();
        }

        let raw_data = ArchivedEntryData::from_entry_data(&record_data);

        let mut record_data_clone =
            MutableEntryData::try_new(raw_data.entry_type().inner()).unwrap();

        for (key, value) in raw_data.fields() {
            record_data_clone
                .check_and_insert(key.inner(), value.inner())
                .unwrap();
        }

        assert_eq!(record_data, record_data_clone);
        assert_eq!(
            raw_data.as_bytes(),
            ArchivedEntryData::from_entry_data(&record_data_clone).as_bytes()
        );
    }

    #[test]
    fn format_consistency() {
        let mut data = MutableEntryData::default();
        let fields = [
            ("author", "Alex Rutar"),
            ("journal", "Journal of Great Papers"),
            ("title", "A wonderful title"),
            ("year", "2036"),
        ];

        for (k, v) in fields {
            data.check_and_insert(k, v).unwrap();
        }

        let archived = ArchivedEntryData::from_entry_data(&data);
        assert!(ArchivedEntryData::validate(archived.as_bytes()).is_ok());
        assert_eq!(
            archived.as_bytes(),
            [
                1, 0, 0, 0, 0, 0, 0, 0, // meta
                4, 0, 0, 0, // entry length 4
                4, 0, 0, 0, // 4 fields
                84, 0, 0, 0, 6, 0, 0, 0, 90, 0, 0, 0, 10, 0, 0, 0, // field 1
                100, 0, 0, 0, 7, 0, 0, 0, 107, 0, 0, 0, 23, 0, 0, 0, // field 2
                130, 0, 0, 0, 5, 0, 0, 0, 135, 0, 0, 0, 17, 0, 0, 0, // field 3
                152, 0, 0, 0, 4, 0, 0, 0, 156, 0, 0, 0, 4, 0, 0, 0, // field 4
                b'm', b'i', b's', b'c', // entry type
                b'a', b'u', b't', b'h', b'o', b'r', // key 1
                b'A', b'l', b'e', b'x', b' ', b'R', b'u', b't', b'a', b'r', // val 1
                b'j', b'o', b'u', b'r', b'n', b'a', b'l', // key 2
                b'J', b'o', b'u', b'r', b'n', b'a', b'l', b' ', b'o', b'f', b' ', b'G', b'r', b'e',
                b'a', b't', b' ', b'P', b'a', b'p', b'e', b'r', b's', // val 2
                b't', b'i', b't', b'l', b'e', // key 3
                b'A', b' ', b'w', b'o', b'n', b'd', b'e', b'r', b'f', b'u', b'l', b' ', b't', b'i',
                b't', b'l', b'e', // val 3
                b'y', b'e', b'a', b'r', // key 4
                b'2', b'0', b'3', b'6' // val 4
            ]
        );

        let data = MutableEntryData::try_new("article").unwrap();

        let archived = ArchivedEntryData::from_entry_data(&data);
        assert!(ArchivedEntryData::validate(archived.as_bytes()).is_ok());
        assert_eq!(
            archived.as_bytes(),
            [
                1, 0, 0, 0, 0, 0, 0, 0, // meta
                7, 0, 0, 0, // entry length 7
                0, 0, 0, 0, // 0 fields
                b'a', b'r', b't', b'i', b'c', b'l', b'e'
            ]
        );
    }
}
