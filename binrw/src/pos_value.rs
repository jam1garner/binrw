use super::*;
use core::fmt;

/// A wrapper where the position it was read from is stored alongside the value
/// ```rust
/// use binrw::{BinRead, PosValue, BinReaderExt, io::Cursor};
///
/// #[derive(BinRead)]
/// struct MyType {
///     a: u16,
///     b: PosValue<u8>
/// }
///
/// let val = Cursor::new(b"\xFF\xFE\xFD").read_be::<MyType>().unwrap();
/// assert_eq!(val.b.pos, 2);
/// assert_eq!(*val.b, 0xFD);
/// ```
pub struct PosValue<T> {
    pub val: T,
    pub pos: u64
}

impl<T: BinRead> BinRead for PosValue<T> {
    type Args = T::Args;

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: T::Args)
        -> BinResult<Self>
    {
        let pos = reader.seek(SeekFrom::Current(0))?;

        Ok(PosValue {
            pos,
            val: T::read_options(reader, options, args)?
        })
    }

    fn after_parse<R: Read + Seek>(&mut self, reader: &mut R, options: &ReadOptions, args: Self::Args)
        -> BinResult<()>
    {
        self.val.after_parse(reader, options, args)
    }
}

impl<T> core::ops::Deref for PosValue<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.val
    }
}

impl<T> core::ops::DerefMut for PosValue<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.val
    }
}

impl<T: fmt::Debug> fmt::Debug for PosValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.val.fmt(f)
    }
}

impl<T: Clone> Clone for PosValue<T> {
    fn clone(&self) -> Self {
        Self {
            val: self.val.clone(),
            pos: self.pos
        }
    }
}

impl<U, T: PartialEq<U>> PartialEq<U> for PosValue<T> {
    fn eq(&self, other: &U) -> bool {
        self.val == *other
    }
}

#[cfg(test)]
mod tests {
    use crate as binrw;

    #[test]
    fn pos_value() {
        use binrw::{BinRead, PosValue, BinReaderExt, io::Cursor};

        #[derive(BinRead)]
        struct MyType {
            a: u16,
            b: PosValue<u8>
        }

        let mut val = Cursor::new(b"\xFF\xFE\xFD").read_be::<MyType>().unwrap();
        assert_eq!(val.a, 0xFFFE);
        assert_eq!(val.b.pos, 2);
        assert_eq!(*val.b, 0xFD);
        assert_eq!(val.b, 0xFDu8);

        *val.b = 1u8;
        assert_eq!(*val.b, 1);
        assert_eq!(format!("{:?}", val.b), "1");
        let clone = val.b.clone();
        assert_eq!(*clone, *val.b);
        assert_eq!(clone.pos, val.b.pos);
    }
}
