use crate::{
    io::{Write, Seek},
    BinResult,
};

pub trait BinWrite {
    fn write<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()> {

    }
}
