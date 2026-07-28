# Autobib entry

This repository contains the implementation for the entry data backend used by the [Autobib](https://github.com/autobib/autobib) bibliography management tool.
Entry data is all of the data contained in a BibTeX entry, excluding the citation key.

This crate defines a zero-copy disk format for entry data.
This allows very efficient access and deserialization of key values directly from disk, without requiring any parsing or allocations (beyond allocating space for the buffer itself).
The implementation here is more compact and has faster reads than a comparable `rkyv`-derived implementation.
It also has the benefit of being fixed and transparently documented.

## Memory layout

The layout of the data in memory is as follows:
```text
| <- HEADER                       -> | <- FIELDS                          -> | <- DATA                 -> |
| meta | entry_type_len | num_fields | (key_idx, key_len, val_idx, val_len)* | entry_type.. keys.. vals.. |
| u64  | u32            | u32        | (u32, u32, u32, u32)*                 | str
```
Brief explanation:

- `meta` is a metadata block, currently set as little-endian bytes to `[1 0 0 0 0 0 0 0]`.
  This distinguishes from the old data format used by Autobib which sets the first byte equal to `0`.
  In the future, Autobib may store additional information in the `meta` block.
- `entry_type_len` is the length of the `entry_type` (`article`, `book`, etc.).
- `num_fields` is the number of fields (`key = {value}` pairs).
- The `FIELDS` block contains the metadata for the fields.
  The raw strings are stored in the `DATA` block, and the indices and lengths refer to sub-strings of the `DATA` block.
  The indices `key_idx` and `val_idx` are absolute.
- The `DATA` block contains all of the strings laid out contiguously in memory, with no gaps.
- All values are restricted in size to `u32::MAX`.
  Moreover, the implementation restricts the size of the entire buffer to `u32::MAX`.
  Autobib uses a SQLite database as the backend, which has even stricter rules on the size of data stored within the database.
- All `u32` and `u64` values are stored in little-endian order.

Benefits of the layout:
- The fields are sorted by key.
  This means that specific `key = {value}` pairs can be found efficiently using [`binary_search_by_key`](https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search_by_key).
- The `DATA` block is a continguous Utf-8 string when valid.
  This improves initial validation since we can check Utf-8 validity in a single pass, rather than check validity for each key and value individually (2-3x slower in benchmarks).
