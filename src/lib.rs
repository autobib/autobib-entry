//! # Autobib entry
//!
//! This crate contains various abstractions over BibTeX entry data, which is all of the data
//! contained in a BibTeX bibliographic record excluding the citation key.
pub mod data;
pub mod error;
pub mod ident;
mod normalize;

#[cfg(feature = "v0")]
pub mod v0;

pub use {
    error::{AccessError, DataError},
    normalize::{Normalization, Normalize},
};
