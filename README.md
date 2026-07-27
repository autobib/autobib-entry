# Autobib entry

This repository contains the implementation for the entry data backend used by the Autobib crate.
Entry data is essentially all of the data contained in a BibTeX entry, ignoring the citation key.

This crate defines a zero-copy disk format for (de)serializing the entry data and for very efficient reading of key values directly from disk without requiring any parsing or allocation.
The implementation here is more compact and has faster reads than a comparable `rkyv`-derived implementation.
It also has the benefit of being fixed and transparently documented.

## Memory layout

The memory layout of the data is as follows:
```text
<- HEADER                -> | <- FIELDS                          -> | <- DATA                 -> |
entry_type_len | num_fields | (key_idx, key_len, val_idx, val_len)* | entry_type.. keys.. vals.. |
u32            | u32        | (u32, u32, u32, u32)*                 |
```
Brief explanation:

- `entry_type_len` is the length of the `entry_type` (`article`, `book`, etc.).
- `num_fields` is the number of fields (`key = {value}` pairs).
- The `FIELDS` block contains the metadata for the fields.
  The raw strings are stored in the `DATA` block, and the indices and lengths refer to sub-slices of the `DATA` block.
  The indices `key_idx` and `val_idx` are absolute.
- The `DATA` block contains all of the strings laid out contiguously in memory, with no gaps.
- All values are restricted in size to `u32::MAX`.
  Moreover, the implementation restricts the size of the entire buffer to `u32::MAX`.
  Autobib uses a SQLite database as the backend, which has even stricter rules on the size of data stored within the database.
- All `u32` values are stored in little-endian order.

The fields are sorted by key.
This means that specific `key = {value}` pairs can be found efficiently using [`binary_search_by_key`](https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search_by_key).
