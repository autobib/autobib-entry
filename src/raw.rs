// # Memory layout
//
// <- HEADER                -> | <- FIELDS                          -> | <- DATA                 -> |
// entry_type_len | num_fields | (key_idx, key_len, val_idx, val_len)* | entry_type.. keys.. vals.. |
// u32            | u32        | (u32, u32, u32, u32)*                 |

use std::{ops::Range, str::from_utf8_unchecked};

use crate::{FieldKeyRef, FieldValueRef, data::EntryData, error::DeserializationError};

pub fn serialize<D: EntryData + ?Sized>(data: &D) -> Box<[u8]> {
    let entry_type_bytes = data.entry_type().inner().as_bytes();
    let num_fields = data.count_fields();

    // pre-compute how much space we need since we will do non-linear allocation
    let header_required = 8;
    let fields_required = 16 * num_fields;
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
    buf[0..4].copy_from_slice(&(entry_type_bytes.len() as u32).to_le_bytes());
    buf[4..8].copy_from_slice(&(data.count_fields() as u32).to_le_bytes());

    // first, the entry data
    let mut offset = data_start;
    buf[offset..offset + entry_type_bytes.len()].copy_from_slice(entry_type_bytes);
    offset = data_start + entry_type_bytes.len();

    // then all of the fields
    for (idx, (k, v)) in data.fields().into_iter().enumerate() {
        let field_start = 8 + 16 * idx;

        // write the field key data and the field key
        buf[field_start..field_start + 4].copy_from_slice(&(offset as u32).to_le_bytes());
        buf[field_start + 4..field_start + 8]
            .copy_from_slice(&(k.inner().len() as u32).to_le_bytes());
        buf[offset..offset + k.inner().len()].copy_from_slice(k.inner().as_bytes());
        offset = offset + k.inner().len();

        // write the field value data and the field value
        buf[field_start + 8..field_start + 12].copy_from_slice(&(offset as u32).to_le_bytes());
        buf[field_start + 12..field_start + 16]
            .copy_from_slice(&(v.inner().len() as u32).to_le_bytes());
        buf[offset..offset + v.inner().len()].copy_from_slice(v.inner().as_bytes());
        offset = offset + v.inner().len();
    }

    buf
}

#[derive(PartialEq)]
#[repr(transparent)]
pub struct RawEntryData([u8]);

impl RawEntryData {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Perform validation required for memory safety.
    pub fn validate(bytes: &[u8]) -> Result<(), DeserializationError> {
        // checking header
        let Some((&[e0, e1, e2, e3, l0, l1, l2, l3], _)) = bytes.split_first_chunk::<8>() else {
            return Err(DeserializationError::IncompleteHeader);
        };
        let entry_type_len = u32::from_le_bytes([e0, e1, e2, e3]) as usize;
        let num_fields = u32::from_le_bytes([l0, l1, l2, l3]) as usize;

        // checking that there is data
        let data_start = 8 + 16 * num_fields;
        let Some(data) = bytes.get(data_start..) else {
            return Err(DeserializationError::IncompleteFields);
        };

        // checking string data is valid utf8
        let data_str = std::str::from_utf8(data)?;

        // checking continguous indices
        let mut kv_data_start = data_start + entry_type_len;

        for idx in 0..num_fields {
            let offset = 8 + 16 * idx;
            // we already checked that these will return valid indices with the length check above
            let (&field_bytes, _) = unsafe {
                bytes
                    .get_unchecked(offset..)
                    .split_first_chunk::<16>()
                    .unwrap_unchecked()
            };
            let (key_idx, key_len, val_idx, val_len) = FieldAccess(field_bytes).parts();

            // check that the indices are contiguous and correspond to valid char boundaries
            if kv_data_start != key_idx {
                return Err(DeserializationError::InvalidIndex(idx));
            }

            if kv_data_start + key_len != val_idx {
                return Err(DeserializationError::InvalidIndex(idx));
            }

            if !data_str.is_char_boundary(key_idx - data_start) {
                return Err(DeserializationError::InvalidStrOffset(idx));
            }

            if !data_str.is_char_boundary(val_idx - data_start) {
                return Err(DeserializationError::InvalidStrOffset(idx));
            }

            kv_data_start = kv_data_start + key_len + val_len;
        }

        // we should end at bytes
        if kv_data_start != bytes.len() {
            return Err(DeserializationError::TrailingBytes(kv_data_start));
        }

        Ok(())
    }

    pub fn load(bytes: Box<[u8]>) -> Result<Box<Self>, DeserializationError> {
        Self::validate(&bytes)?;
        unsafe { Ok(Self::load_unchecked(bytes)) }
    }

    pub unsafe fn load_unchecked(buf: Box<[u8]>) -> Box<Self> {
        unsafe { Box::from_raw(Box::into_raw(buf) as *mut RawEntryData) }
    }

    pub fn access(bytes: &[u8]) -> Result<&Self, DeserializationError> {
        Self::validate(&bytes)?;
        unsafe { Ok(Self::access_unchecked(bytes)) }
    }

    pub unsafe fn access_unchecked(b: &[u8]) -> &Self {
        unsafe { std::mem::transmute(b) }
    }

    pub fn from_entry_data<D: EntryData + ?Sized>(data: &D) -> Box<RawEntryData> {
        unsafe { RawEntryData::load_unchecked(serialize(data)) }
    }
}

struct RawLayout {
    entry_type_len: usize,
    num_fields: usize,
    data_start: usize,
}

impl RawLayout {
    const FIELDS_START: usize = 8;

    fn entry_type_range(&self) -> Range<usize> {
        self.data_start..self.data_start + self.entry_type_len
    }

    fn all_fields_range(&self) -> Range<usize> {
        Self::FIELDS_START..self.data_start
    }
}

#[derive(Clone, Copy)]
struct FieldAccess([u8; 16]);

impl FieldAccess {
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

    #[inline]
    fn access_in<'r>(self, data: &'r [u8]) -> (FieldKeyRef<'r>, FieldValueRef<'r>) {
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

impl RawEntryData {
    #[inline]
    fn layout(&self) -> RawLayout {
        let (&[e0, e1, e2, e3, n0, n1, n2, n3], _) =
            unsafe { self.0.split_first_chunk::<8>().unwrap_unchecked() };
        let entry_type_len = u32::from_le_bytes([e0, e1, e2, e3]) as usize;
        let num_fields = u32::from_le_bytes([n0, n1, n2, n3]) as usize;
        let data_start = 8 + 16 * num_fields;
        RawLayout {
            entry_type_len,
            num_fields,
            data_start,
        }
    }

    #[inline]
    fn raw_fields(&self) -> &[[u8; 16]] {
        let ly = self.layout();
        unsafe {
            self.0
                .get_unchecked(ly.all_fields_range())
                .as_chunks_unchecked()
        }
    }
}

impl crate::data::EntryData for RawEntryData {
    fn fields(&self) -> impl IntoIterator<Item = (FieldKeyRef<'_>, FieldValueRef<'_>)> {
        let rf = self.raw_fields();
        rf.iter()
            .map(|chunk| FieldAccess(*chunk).access_in(&self.0))
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
        let rf = self.raw_fields();
        rf.binary_search_by_key(&field_name, |&chunk| {
            FieldAccess(chunk).access_in(&self.0).0.inner()
        })
        .ok()
        .map(|idx| unsafe { FieldAccess(*rf.get_unchecked(idx)).access_in(&self.0).1 })
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

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

        let serialized = RawEntryData::from_entry_data(&data);
        assert!(RawEntryData::validate(serialized.as_bytes()).is_ok());
        assert_eq!(
            serialized.as_bytes(),
            [
                4, 0, 0, 0, // entry length 4
                4, 0, 0, 0, // 4 fields
                76, 0, 0, 0, 6, 0, 0, 0, 82, 0, 0, 0, 10, 0, 0, 0, // field 1
                92, 0, 0, 0, 7, 0, 0, 0, 99, 0, 0, 0, 23, 0, 0, 0, // field 2
                122, 0, 0, 0, 5, 0, 0, 0, 127, 0, 0, 0, 17, 0, 0, 0, // field 3
                144, 0, 0, 0, 4, 0, 0, 0, 148, 0, 0, 0, 4, 0, 0, 0, // field 4
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

        let serialized = RawEntryData::from_entry_data(&data);
        assert!(RawEntryData::validate(serialized.as_bytes()).is_ok());
        assert_eq!(
            serialized.as_bytes(),
            [
                7, 0, 0, 0, 0, 0, 0, 0, b'a', b'r', b't', b'i', b'c', b'l', b'e'
            ]
        );
    }

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

        let serialized = RawEntryData::from_entry_data(&data);
        assert_eq!(serialized.count_fields(), fields.len());
        for (k, v) in fields {
            assert_eq!(serialized.get_field_str(k), Some(v));
        }
        for ((k, v), (ser_k, ser_v)) in fields.iter().zip(serialized.fields()) {
            assert_eq!(k, &ser_k.inner());
            assert_eq!(v, &ser_v.inner());
        }
    }
}
