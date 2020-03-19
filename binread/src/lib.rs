//! A Rust crate for helping parse binary data using ✨macro magic✨
#![cfg_attr(not(feature="std"), no_std)]

#[cfg(feature = "std")]
use std as alloc;

#[cfg(not(feature = "std"))]
extern crate alloc;


#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    vec::Vec,
    string::String,
};

pub mod io;
pub mod error;
pub mod endian;
pub mod file_ptr;
pub mod options;
pub mod strings;

#[cfg(feature = "std")]
#[cfg(feature = "debug_template")]
pub mod binary_template;

use core::any::{Any, TypeId};
pub use error::Error;
pub use endian::Endian;
pub use file_ptr::FilePtr;
pub use options::{ReadOptions, AfterParseOptions, Imports};
pub use strings::{NullString, NullWideString};

use io::{Read, Seek, SeekFrom};

/// Derive macro for BinRead. [Usage here](BinRead).
pub use binread_derive::BinRead;

mod binread_impls;
pub use binread_impls::*;

pub type BinResult<T> = core::result::Result<T, Error>;

pub trait BinRead: Sized {
    type Args: Any + Copy;

    fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        let args = match Self::args_default() {
            Some(args) => args,
            None => panic!("Must pass args, no args_default implemented")
        };

        Self::read_options(reader, &ReadOptions::default(), args)
    }
    
    fn read_args<R: Read + Seek>(reader: &mut R, args: Self::Args) -> BinResult<Self> {
        Self::read_options(reader, &ReadOptions::default(), args)
    }

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self>;

    fn after_parse<R: Read + Seek>(&mut self, _: &mut R, _: &ReadOptions, _: Self::Args, _: &AfterParseOptions) -> BinResult<()> {
        Ok(())
    }

    fn args_default() -> Option<Self::Args> {
        if TypeId::of::<Self::Args>() == TypeId::of::<()>() {
            Some(*unsafe{
                core::mem::transmute::<_, &Self::Args>(&())
            })
        } else {
            None
        }
    }
}
