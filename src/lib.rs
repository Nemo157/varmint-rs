#![feature(conservative_impl_trait)]

extern crate arrayvec;
extern crate futures;
#[macro_use]
extern crate tokio_core;

mod read;
mod write;

pub use read::{
    read_u64_varint,
    try_read_u64_varint,
    read_usize_varint,
    try_read_usize_varint,
};

pub use write::{
    write_u64_varint,
    write_usize_varint,
};
