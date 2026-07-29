//! # Autobib entry
//!
//! This crate contains various abstractions over BibTeX entry data, which is all of the data
//! contained in a BibTeX bibliographic record excluding the citation key.
//!
//! ## Data abstractions and formats
//!
//! This crate exposes two key abstractions: [`EntryData`], representing types which contain all of
//! the data in an entry, and [`Archive`], representing types which can be serialized to raw bytes
//! and which support zero-copy access and deserialization.
//!
//! For entry data which supports addition and deletion of fields and efficient access, use
//! [`data::MutableEntryData`]. The [`data`] module also defines a large number of operations and
//! normalizations on entry data.
//!
//! In order to use [`Archive`] implementations, you need to enable one of the `v*` feature flags.
//! All versions are loaded by default.
//!
//! - [`v1`] is the current format, with very fast reads, writes, and serialization (2-3x faster
//!   than equivalent `rkyv 0.8`-derived implementations for normal entry data sizes).
//! - [`v0`] is the legacy format, which is used by older Autobib databases. It is (slightly) more
//!   compact, but has rather slow reads; most egregiously, it has `O(n)` single field access.
//!
//! The individual [`Archive`] implementations cannot read from data serialized in other formats.
//! However, you can use [`MutableEntryData::from_archive_universal`](data::MutableEntryData::from_archive_universal)
//! to read from any data format for which the corresponding feature is enabled.
//!
//! ## Identifier abstractions
//!
//! Identifier abstractions are in the [`ident`] module, including the entry type, field keys, and
//! field values.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

pub mod data;
pub mod error;
pub mod ident;

#[cfg(feature = "v0")]
#[cfg_attr(docsrs, doc(cfg(feature = "v0")))]
pub mod v0;

#[cfg(feature = "v1")]
#[cfg_attr(docsrs, doc(cfg(feature = "v1")))]
pub mod v1;

pub use {
    data::{Archive, EntryData},
    error::{AccessError, DataError},
};
