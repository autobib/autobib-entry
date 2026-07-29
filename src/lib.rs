//! # Autobib entry
//!
//! This crate contains various abstractions over BibTeX entry data, which is all of the data
//! contained in a BibTeX bibliographic record excluding the citation key.
//!
//! ## Main entry points
//!
//! Identifier abstractions are in the [`ident`] module, including the entry type, field keys, and
//! field values.
pub mod data;
pub mod error;
pub mod ident;
mod normalize;

#[cfg(feature = "v0")]
pub mod v0;

#[cfg(feature = "v1")]
pub mod v1;

pub use {
    data::{Archive, EntryData},
    error::{AccessError, DataError},
    normalize::{Normalization, Normalize},
};
