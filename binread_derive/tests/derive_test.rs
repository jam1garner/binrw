#![allow(dead_code)]
use binwrite::*;
use binwrite_derive::BinWrite;
use std::ops::Add;
use crc::crc32::checksum_ieee as crc32;

// Easy to add, reusable components
// vec_with_len::write uses the format of:
// [len: u32][vec[0]: T][vec[1]: T][...][vec[n]: T]
mod vec_with_len {
    use std::io::{Write, Result};
    use binwrite::{WriterOption, BinWrite};

    pub fn write<W, T>(vec: &Vec<T>, writer: &mut W, options: &WriterOption) -> Result<()>
        where W: Write,
              T: BinWrite,
    {
        BinWrite::write_options(&(vec.len() as u32), writer, options)?;
        BinWrite::write_options(vec, writer, options)
    }
}

pub fn add<T: Add<Output = T> + Copy>(lhs: T) -> impl Fn(T) -> T {
    move |rhs| lhs + rhs
}

#[derive(BinWrite)]
#[binwrite(little)]
struct Test {
    magic: [char; 4],
    
    #[binwrite(with(vec_with_len::write))]
    nums: Vec<u64>,
    
    #[binwrite(big)]
    val_big: u32,
    
    #[binwrite(little)]
    val_little: u32,
    
    #[binwrite(preprocessor(add(2)))]
    val_u8: u8,

    #[binwrite(ignore)]
    this_will_be_ignored: u32,

    #[binwrite(align(0x8), cstr, align_after(0x10), postprocessor(|v: Vec<u8>|{
        (crc32(&v[..]), v)
    }))]
    test: String,

    tuple_test: (u32, String, u8),
}

#[test]
fn main() {
    let mut bytes = vec![];
    let test = Test {
        magic: ['T', 'E', 'S', 'T'],
        nums: vec![0, 1, 2, 3],
        val_big: 0x1234_5678,
        val_little: 0x1234_5678,
        val_u8: 0x69,
        this_will_be_ignored: 0x42042000,
        test: "this_is_test".to_string(),
        tuple_test: (0xBADF00D5, "tuple test".into(), 0x33)
    };
    
    test.write(&mut bytes).unwrap();

    println!("\n\nResult:");
    hex::print_bytes(&bytes);
    println!("\n\n\n");

    let mut bytes = vec![];
    let test2 = Test2 {
        field2: 3,
        body: "test".to_string()
    };

    test2.write(&mut bytes).unwrap();
    
    println!("\n\nResult:");
    hex::print_bytes(&bytes);
    println!("\n\n\n");
}

#[derive(BinWrite)]
#[binwrite(big)]
struct Test2 {
    // this field comes after the magic and crc32 and before the body of the file
    #[binwrite(ignore)]
    field2: u32,

    #[binwrite(postprocessor(|body: Vec<u8>|{
        (
            "MAG0", // File magic
            crc32(&body[..]),
            self.field2,
            body
        )
    }))]
    body: String
}


mod hex {
    pub fn print_bytes(vec: &Vec<u8>) {
        vec.chunks(0x10)
            .for_each(|line|{
                line.iter()
                    .enumerate()
                    .for_each(|(i, byte)|{
                        if i % 0x4 == 0 && i != 0 {
                            print!(" ");
                        }
                        print!("{:02X} ", byte);
                    });
                println!("");
            });
    }
}
