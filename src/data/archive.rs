//! # Zero-copy deserialization format
//!
//! ## Memory format
//!
//! All `u32` values are stored in little-endian order.
//! ```text
//! | <- HEADER      -> | <- TYPE -> | <- FIELDS                          -> | <- DATA                 -> |
//! | meta | num_fields | (idx, len) | (key_idx, key_len, val_idx, val_len)* | entry_type.. keys.. vals.. |
//! | u32  | u32        | [u32, u32] | [u32, u32, u32, u32]*                 | str
//! ```
//!
//! ### Format explanation
//!
//! - `HEADER`: fixed-size metadata for the data
//!   - `meta`: a currently unused metadata block, currently set as little-endian bytes to `[1 0 0 0 0 0 0 0]`.
//!     This distinguishes from the old data format used by Autobib which sets the first byte equal to `0`.
//!     For validity, only the first byte is checked.
//!     Future versions of this binary format may store additional metadata in the `meta` block.
//!   - `num_fields`: the number of `key = {value}` fields
//! - `TYPE`: pointer to the entry type
//!   - `idx`: an index into this byte buffer indicating the start of the entry type
//!   - `len`: the length of the entry type
//! - `FIELDS`: variable-size metadata for each `key = {value}` field
//!   - `key_idx`: an index into this byte buffer indicating the start of the `key`
//!   - `key_len`: the length of the `key`
//!   - `val_idx`: an index into this byte buffer indicating the start of the `value`
//!   - `val_len`: the length of the `value`
//! - `DATA`: a contiguous string storing the raw contents of the entry type, and the field keys and the values.
//!   The pointers in `TYPE` and `FIELDS` refer to valid sub-strings of the `DATA` block.
//!
//! ### Format features
//!
//! - The fields are sorted by key.
//!   This means that specific `key = {value}` pairs can be found efficiently using [`binary_search_by_key`](https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search_by_key).
//! - The `DATA` block is a continguous Utf-8 string when valid.
//!   This improves initial validation since we can check Utf-8 validity in a single pass, rather than check validity for each key and value individually (2-3x slower in benchmarks).

use std::str::from_utf8_unchecked;

const HEADER_LEN: usize = 16;
const FIELD_LEN: usize = 16;

use crate::{
    data::EntryData,
    error::AccessError,
    ident::{EntryTypeRef, FieldKeyRef, FieldValueRef},
};

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
        "Cannot write entry data exceeding 2^32 bytes!"
    );

    // initialize as zeroed; we will write non-sequentially
    let buf = Box::new_zeroed_slice(raw_data_len);
    let mut buf = unsafe { buf.assume_init() };

    // HEADER
    unsafe {
        *buf.get_unchecked_mut(0) = 1; // recall other values are zeroed
        buf.get_unchecked_mut(4..8)
            .copy_from_slice(&(num_fields as u32).to_le_bytes());
        buf.get_unchecked_mut(8..12)
            .copy_from_slice(&(data_start as u32).to_le_bytes()); // we always write entry type
        // first
        buf.get_unchecked_mut(12..16)
            .copy_from_slice(&(entry_type_bytes.len() as u32).to_le_bytes());
    }

    // first, the entry data
    let mut offset = data_start; // a cursor for the current position within the buffer up to which
    // we have written
    unsafe {
        buf.get_unchecked_mut(offset..offset + entry_type_bytes.len())
            .copy_from_slice(entry_type_bytes)
    };
    offset = data_start + entry_type_bytes.len();

    // then all of the fields
    for (idx, (k, v)) in data.fields().into_iter().enumerate() {
        let field_start = HEADER_LEN + FIELD_LEN * idx;

        // write the field key data and the field key
        unsafe {
            buf.get_unchecked_mut(field_start..field_start + 4)
                .copy_from_slice(&(offset as u32).to_le_bytes());
            buf.get_unchecked_mut(field_start + 4..field_start + 8)
                .copy_from_slice(&(k.inner().len() as u32).to_le_bytes());
            buf.get_unchecked_mut(offset..offset + k.inner().len())
                .copy_from_slice(k.inner().as_bytes());
        }
        offset += k.inner().len();

        // write the field value data and the field value
        unsafe {
            buf.get_unchecked_mut(field_start + 8..field_start + 12)
                .copy_from_slice(&(offset as u32).to_le_bytes());
            buf.get_unchecked_mut(field_start + 12..field_start + 16)
                .copy_from_slice(&(v.inner().len() as u32).to_le_bytes());
            buf.get_unchecked_mut(offset..offset + v.inner().len())
                .copy_from_slice(v.inner().as_bytes());
        }
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
        let Some((&[1, 0, 0, 0, l0, l1, l2, l3, _, _, _, _, e0, e1, e2, e3], _)) =
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
        // now it is guaranteed that bytes.len() >= HEADER_LEN + FIELD_LEN * num_fields

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
            let (key, val) = FieldAccess(field_bytes).parts();

            // check that the indices are contiguous and correspond to valid char boundaries
            if kv_data_start != key.idx as usize {
                return Err(AccessError::InvalidIndex(idx));
            }

            if kv_data_start + key.len as usize != val.idx as usize {
                return Err(AccessError::InvalidIndex(idx));
            }

            if !data_str.is_char_boundary(key.idx as usize - data_start) {
                return Err(AccessError::InvalidStrOffset(idx));
            }

            if !data_str.is_char_boundary(val.idx as usize - data_start) {
                return Err(AccessError::InvalidStrOffset(idx));
            }

            kv_data_start = kv_data_start + key.len as usize + val.len as usize;
        }

        // no trailing bytes
        if kv_data_start != bytes.len() {
            return Err(AccessError::TrailingBytes(kv_data_start));
        }

        Ok(())
    }

    /// Load the provided byte buffer, first checking that the underlying bytes are valid.
    #[inline]
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
    #[inline]
    pub unsafe fn load_unchecked(buf: Box<[u8]>) -> Box<Self> {
        unsafe { Box::from_raw(Box::into_raw(buf) as *mut ArchivedEntryData) }
    }

    /// Access data from the provided byte buffer without any copying or parsing, first
    /// checking that the underlying bytes are valid.
    #[inline]
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
    #[inline]
    pub unsafe fn access_unchecked(b: &[u8]) -> &Self {
        unsafe { std::mem::transmute(b) }
    }

    /// Construct the byte representation from any entry data implementation.
    #[inline]
    pub fn from_entry_data<D: EntryData + ?Sized>(data: &D) -> Box<ArchivedEntryData> {
        unsafe { ArchivedEntryData::load_unchecked(archive(data)) }
    }

    /// Convert into boxed bytes.
    #[inline]
    pub fn into_boxed_bytes(self: Box<Self>) -> Box<[u8]> {
        unsafe { Box::from_raw(Box::into_raw(self) as *mut [u8]) }
    }
}

#[derive(Debug, Clone, Copy)]
struct StrPtr {
    idx: u32,
    len: u32,
}

impl StrPtr {
    #[inline]
    unsafe fn read_unchecked<'r>(self, data: &'r [u8]) -> &'r str {
        let idx = self.idx as usize;
        let len = self.len as usize;
        unsafe { from_utf8_unchecked(data.get_unchecked(idx..idx + len)) }
    }
}

#[derive(Debug, Clone, Copy)]
struct RawHeader {
    #[expect(unused)]
    meta: [u8; 4],
    num_fields: u32,
}

impl RawHeader {
    #[inline]
    unsafe fn load_unchecked(buf: &[u8]) -> Self {
        let &[m0, m1, m2, m3, l0, l1, l2, l3] =
            unsafe { buf.get_unchecked(0..8).as_array::<8>().unwrap_unchecked() };
        let meta = [m0, m1, m2, m3];
        let num_fields = u32::from_le_bytes([l0, l1, l2, l3]);
        Self { meta, num_fields }
    }

    #[inline]
    unsafe fn read_fields_unchecked(self, buf: &[u8]) -> &[[u8; FIELD_LEN]] {
        unsafe {
            buf.get_unchecked(HEADER_LEN..HEADER_LEN + FIELD_LEN * self.num_fields as usize)
                .as_chunks_unchecked()
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RawType {
    entry_type: StrPtr,
}

impl RawType {
    #[inline]
    unsafe fn load_unchecked(buf: &[u8]) -> Self {
        let &[i0, i1, i2, i3, l0, l1, l2, l3] =
            unsafe { buf.get_unchecked(8..16).as_array::<8>().unwrap_unchecked() };
        let idx = u32::from_le_bytes([i0, i1, i2, i3]);
        let len = u32::from_le_bytes([l0, l1, l2, l3]);
        Self {
            entry_type: StrPtr { idx, len },
        }
    }
}

/// An accessor for a single field.
#[derive(Debug, Clone, Copy)]
struct FieldAccess([u8; FIELD_LEN]);

impl FieldAccess {
    /// Split the field into its constituent `usize` parts: `key_idx`, `key_len`, `val_id`, and
    /// `val_len`.
    #[inline]
    fn parts(self) -> (StrPtr, StrPtr) {
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
            StrPtr {
                idx: u32::from_le_bytes([ki0, ki1, ki2, ki3]),
                len: u32::from_le_bytes([kl0, kl1, kl2, kl3]),
            },
            StrPtr {
                idx: u32::from_le_bytes([vi0, vi1, vi2, vi3]),
                len: u32::from_le_bytes([vl0, vl1, vl2, vl3]),
            },
        )
    }

    /// Access the contents of this field in the data buffer.
    ///
    /// # Safety
    ///
    /// The data buffer must be valid for the field from which this struct was constructed.
    #[inline]
    unsafe fn access_in<'r>(self, data: &'r [u8]) -> (FieldKeyRef<'r>, FieldValueRef<'r>) {
        let (key_ptr, val_ptr) = self.parts();
        unsafe {
            (
                FieldKeyRef(key_ptr.read_unchecked(data)),
                FieldValueRef(val_ptr.read_unchecked(data)),
            )
        }
    }
}

impl ArchivedEntryData {
    /// Obtain the field metadata as a slice of `Field`s.
    #[inline]
    fn raw_fields(&self) -> &[[u8; FIELD_LEN]] {
        unsafe { RawHeader::load_unchecked(&self.0).read_fields_unchecked(&self.0) }
    }

    /// Obtain the field metadata as a slice of `Field`s.
    #[inline]
    fn raw_entry_type(&self) -> &str {
        unsafe {
            RawType::load_unchecked(&self.0)
                .entry_type
                .read_unchecked(&self.0)
        }
    }

    #[inline]
    fn num_fields(&self) -> usize {
        unsafe { RawHeader::load_unchecked(&self.0).num_fields as _ }
    }
}

impl EntryData for ArchivedEntryData {
    fn fields(&self) -> impl IntoIterator<Item = (FieldKeyRef<'_>, FieldValueRef<'_>)> {
        unsafe {
            self.raw_fields()
                .iter()
                .map(|chunk| FieldAccess(*chunk).access_in(&self.0))
        }
    }

    fn entry_type(&self) -> EntryTypeRef<'_> {
        EntryTypeRef(self.raw_entry_type())
    }

    fn count_fields(&self) -> usize {
        self.num_fields()
    }

    fn get_field<'r>(&'r self, field_name: &str) -> Option<FieldValueRef<'r>> {
        if self.num_fields() <= 6 {
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

impl EntryData for Box<ArchivedEntryData> {
    fn fields(&self) -> impl IntoIterator<Item = (FieldKeyRef<'_>, FieldValueRef<'_>)> {
        self.as_ref().fields()
    }

    fn entry_type(&self) -> EntryTypeRef<'_> {
        self.as_ref().entry_type()
    }

    fn count_fields(&self) -> usize {
        self.as_ref().count_fields()
    }

    fn get_field<'r>(&'r self, field_name: &str) -> Option<FieldValueRef<'r>> {
        self.as_ref().get_field(field_name)
    }

    fn get_field_str<'r>(&'r self, field_name: &str) -> Option<&'r str> {
        self.as_ref().get_field_str(field_name)
    }

    fn contains_field(&self, field_name: &str) -> bool {
        self.as_ref().contains_field(field_name)
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
            data.try_insert(k, v).unwrap();
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
            record_data.try_insert(k, v).unwrap();
        }

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
    fn format_consistency() {
        let mut data = MutableEntryData::default();
        let fields = [
            ("author", "Alex Rutar"),
            ("journal", "Journal of Great Papers"),
            ("title", "A wonderful title"),
            ("year", "2036"),
        ];

        for (k, v) in fields {
            data.try_insert(k, v).unwrap();
        }

        let archived = ArchivedEntryData::from_entry_data(&data);
        assert!(ArchivedEntryData::validate(archived.as_bytes()).is_ok());
        assert_eq!(
            archived.as_bytes(),
            [
                1, 0, 0, 0, // meta
                4, 0, 0, 0, // 4 fields
                80, 0, 0, 0, 4, 0, 0, 0, // entry type
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
                1, 0, 0, 0, // meta
                0, 0, 0, 0, // 0 fields
                16, 0, 0, 0, 7, 0, 0, 0, // entry type
                b'a', b'r', b't', b'i', b'c', b'l', b'e'
            ]
        );
    }
}
