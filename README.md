# Autobib entry

This repository contains the implementation for the entry data backend used by the [Autobib](https://github.com/autobib/autobib) bibliography management tool.
Entry data is all of the data contained in a BibTeX entry, excluding the citation key.

This crate defines zero-copy disk formats for entry data.
This allows very efficient access and deserialization of key values directly from disk, without requiring any parsing or allocations (beyond allocating space for the buffer itself).
The main `v1` implementation here is more compact and has faster reads than a comparable `rkyv`-derived implementation.
It also has the benefit of being fixed and transparently documented.

There are multiple formats:

- [`v1`](#v1-memory-layout) - current format
- [`v0`](#v0-memory-layout) - legacy format used by Autobib databases `<= 4`, versions `< 0.7.0` (slightly more compact, but with much slower reads, in particular `O(n)` field reads)

The format of the data can be determined by the first byte (value `0` corresponds to `v0`, etc.)

## `v1` memory layout

The layout of the data in memory is as follows:
```text
| <- HEADER      -> | <- TYPE -> | <- FIELDS                          -> | <- DATA                 -> |
| meta | num_fields | (idx, len) | (key_idx, key_len, val_idx, val_len)* | entry_type.. keys.. vals.. |
| u32  | u32        | [u32, u32] | [u32, u32, u32, u32]*                 | str
```
All `u32` are stored as little-endian bytes.

### Format explanation

- `HEADER`: fixed-size metadata for the data
  - `meta`: a currently unused metadata block, currently set as little-endian bytes to `[1 0 0 0]`.
    For validity, only the first byte is checked.
    Future versions of this binary format may store additional metadata in the `meta` block.
  - `num_fields`: the number of `key = {value}` fields
- `TYPE`: pointer to the entry type
  - `idx`: an index into this byte buffer indicating the start of the entry type
  - `len`: the length of the entry type
- `FIELDS`: variable-size metadata for each `key = {value}` field
  - `key_idx`: an index into this byte buffer indicating the start of the `key`
  - `key_len`: the length of the `key`
  - `val_idx`: an index into this byte buffer indicating the start of the `value`
  - `val_len`: the length of the `value`
- `DATA`: a contiguous string storing the raw contents of the entry type, and the field keys and the values.
  The pointers in `TYPE` and `FIELDS` refer to valid sub-strings of the `DATA` block.

### Format features

- The fields are sorted by key.
  This means that specific `key = {value}` pairs can be found efficiently using [`binary_search_by_key`](https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search_by_key).
- The `DATA` block is a continguous Utf-8 string when valid.
  This improves initial validation since we can check Utf-8 validity in a single pass, rather than check validity for each key and value individually (2-3x slower in benchmarks).
- The `u32` reads are all aligned.
  This improves read performance substantially, as long as the underlying byte buffer is `u32`-aligned.
  The field metadata is also `u128`-aligned, though this is not currently exploited by the implementation.
- The values do not need to strictly be contiguous, as long as the gaps in between are padded by null bytes.
  For example, this means that fields can be deleted by zeroing-out the higher field keys and overwriting `num_fields`.
  Of course, extra space introduces additional memory and validation overhead and therefore should be avoided if possible.
  The default serialized format will pack the (key, value) pairs contiguously in the same order as specified by the fields.

### Future extensions?

- Allow storing the fields not sorted by key.
  Rearrangement of field keys and field values is permitted since the indices and lengths are absolute and therefore remain valid.
- Store if `DATA` is ASCII for faster initialization.
- Allow 'unpacked' versions with flag (packed default), in which case need to check validity of char boundaries on both sides.
- Add basic in-place mutation.

## `v0` memory layout

The data is stored as a sequence of blocks.
```txt
HEADER | TYPE | DATA*
```
The `HEADER` is a single byte which is `0`.
The `TYPE` consists of
```txt
| entry_type_len | entry_type |
| u8             | str        |
```
Here, `entry_type_len` is the length of `entry_type`.
Then, each block `DATA` is of the form
```txt
| key_len | value_len | key | value |
| u8      | u16       | str | str   |
```
where `key_len` is the length of the first `key` segment, and the `value_len` is the length of the `value` segment.
The `value_len` is encoded as little-endian bytes.
