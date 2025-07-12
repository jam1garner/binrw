#![no_implicit_prelude]

mod t {
    extern crate alloc;
    pub(super) use ::core::prelude::rust_2021::*;
    pub(super) use ::core::{assert_eq, matches, panic, write};
    pub(super) use alloc::{
        format,
        string::{self, String, ToString},
        vec,
        vec::Vec,
    };
}

mod binwrite_temp;
mod r#enum;
mod fn_helper;
mod map_args;
mod r#struct;
mod struct_generic;
mod struct_map;
mod unit_enum;
mod unit_struct;
mod write;
