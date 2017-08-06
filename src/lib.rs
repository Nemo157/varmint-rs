#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_results)]
#![warn(variant_size_differences)]

//! A Rust implementation of the varint format as used in Google's Protocol
//! Buffers.
//!
//! Has two different features for encoding/decoding varints:
//! 
//! The `io` feature (on by default) adds `read_*`/`write_*` methods for
//! various sizes of varints on top of the standard IO traits.
//!
//! The `bytes` feature adds `get_*` methods for various sizes of varints on
//! top of the [`bytes`](https://crates.io/crates/bytes) crate's `Buf` trait.

#[cfg(feature = "bytes")]
extern crate bytes;

mod error;
mod len;
mod parser;

#[cfg(feature = "io")]
mod read;
#[cfg(feature = "io")]
mod write;

#[cfg(feature = "bytes")]
mod bytes_impl;

pub use error::{Error, Result};
pub use len::{len_u64_varint, len_usize_varint};

#[cfg(feature = "io")]
pub use read::ReadVarInt;
#[cfg(feature = "io")]
pub use write::WriteVarInt;

#[cfg(feature = "bytes")]
pub use bytes_impl::BufVarInt;
