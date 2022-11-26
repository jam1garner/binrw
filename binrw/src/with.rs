//! Wrapper type for conversions.

use super::{io, BinRead, BinResult, BinWrite, Endian, ReadFrom, WriteInto};
use crate::meta::{EndianKind, ReadEndian, WriteEndian};
use core::{cmp::Ordering, marker::PhantomData};

/// A wrapper for reading or writing types through a converter.
///
/// The converter must implement [`ReadFrom<T>`] for reads and [`WriteInto<T>`]
/// for writes.
pub struct With<C, T>(PhantomData<C>, T);

impl<C, T> With<C, T> {
    /// Consumes this wrapper, returning the wrapped value.
    pub fn into_inner(self) -> T {
        self.1
    }
}

impl<C, T> From<T> for With<C, T> {
    fn from(value: T) -> Self {
        Self(PhantomData, value)
    }
}

impl<C, T> Clone for With<C, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(PhantomData, self.1.clone())
    }
}

impl<C, T> Copy for With<C, T> where T: Copy {}

impl<C, T> Eq for With<C, T> where T: Eq {}

impl<C, T> Ord for With<C, T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

impl<C, T> PartialEq for With<C, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<C, T> PartialOrd for With<C, T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

impl<C, T> core::fmt::Debug for With<C, T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("With").field(&self.1).finish()
    }
}

impl<C, T> core::ops::Deref for With<C, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<C, T> core::ops::DerefMut for With<C, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

impl<C, T> BinRead for With<C, T>
where
    C: 'static,
    T: ReadFrom<C> + 'static,
{
    type Args = <T as ReadFrom<C>>::Args;

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<Self> {
        <T as ReadFrom<C>>::read_from(reader, endian, args).map(Self::from)
    }
}

impl<C, T> ReadEndian for With<C, T>
where
    C: ReadEndian,
{
    const ENDIAN: EndianKind = C::ENDIAN;
}

impl<C, T> BinWrite for With<C, T>
where
    T: WriteInto<C>,
{
    type Args = <T as WriteInto<C>>::Args;

    fn write_options<W: io::Write + io::Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<()> {
        <T as WriteInto<C>>::write_into(self, writer, endian, args)
    }
}

impl<C, T> WriteEndian for With<C, T>
where
    C: WriteEndian,
{
    const ENDIAN: EndianKind = C::ENDIAN;
}
