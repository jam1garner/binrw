//! Traits representing basic properties of types.

use crate::Endian;
use alloc::{boxed::Box, vec::Vec};
use core::marker::PhantomData;

/// Types that require a magic number when parsed.
///
/// This trait is automatically defined on derived types with a
/// [magic directive](crate::docs::attribute#magic).
pub trait ReadMagic {
    /// The type of the magic number.
    type MagicType;

    /// The magic number.
    const MAGIC: Self::MagicType;
}

/// Types that write a magic number when serialised.
///
/// This trait is automatically defined on derived types with a
/// [magic directive](crate::docs::attribute#magic).
pub trait WriteMagic {
    /// The type of the magic number.
    type MagicType;

    /// The magic number.
    const MAGIC: Self::MagicType;
}

/// Types with explicit read endianness.
///
/// This trait is automatically defined on derived types with a
/// [byte order directive](crate::docs::attribute#byte-order).
pub trait ReadEndian {
    /// The endianness of the type.
    const ENDIAN: EndianKind;
}

/// Types with explicit write endianness.
///
/// This trait is automatically defined on derived types with a
/// [byte order directive](crate::docs::attribute#byte-order).
pub trait WriteEndian {
    /// The endianness of the type.
    const ENDIAN: EndianKind;
}

/// The kind of endianness used by a type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndianKind {
    /// The type has no endianness at all.
    None,
    /// The type uses a fixed endianness.
    Endian(Endian),
    /// The type uses an endianness that is dynamically determined at runtime
    /// from an expression.
    Runtime,
    /// The type uses a heterogenous mix of endianness.
    Mixed,
}

impl EndianKind {
    /// Returns the fixed endianness of the type, if one exists.
    #[must_use]
    pub fn endian(self) -> Option<Endian> {
        match self {
            EndianKind::None => Some(crate::Endian::Native),
            EndianKind::Endian(endian) => Some(endian),
            EndianKind::Runtime | EndianKind::Mixed => None,
        }
    }
}

macro_rules! endian_impl {
    ($($($Ty:ty)+ => $kind:expr),+ $(,)?) => {$($(
        impl ReadEndian for $Ty {
            const ENDIAN: EndianKind = $kind;
        }

        impl WriteEndian for $Ty {
            const ENDIAN: EndianKind = $kind;
        }
    )+)+}
}

endian_impl!(() i8 u8 core::num::NonZeroU8 core::num::NonZeroI8 crate::strings::NullString => EndianKind::None);

impl<T: ReadEndian + ?Sized> ReadEndian for Box<T> {
    const ENDIAN: EndianKind = <T as ReadEndian>::ENDIAN;
}

impl<T: WriteEndian + ?Sized> WriteEndian for Box<T> {
    const ENDIAN: EndianKind = <T as WriteEndian>::ENDIAN;
}

impl<T: ReadEndian> ReadEndian for [T] {
    const ENDIAN: EndianKind = <T as ReadEndian>::ENDIAN;
}

impl<T: WriteEndian> WriteEndian for [T] {
    const ENDIAN: EndianKind = <T as WriteEndian>::ENDIAN;
}

impl<T: ReadEndian, const N: usize> ReadEndian for [T; N] {
    const ENDIAN: EndianKind = <T as ReadEndian>::ENDIAN;
}

impl<T: WriteEndian, const N: usize> WriteEndian for [T; N] {
    const ENDIAN: EndianKind = <T as WriteEndian>::ENDIAN;
}

macro_rules! endian_generic_impl {
    ($($Ty:ident)+) => {$(
        impl<T: ReadEndian> ReadEndian for $Ty<T> {
            const ENDIAN: EndianKind = <T as ReadEndian>::ENDIAN;
        }

        impl<T: WriteEndian> WriteEndian for $Ty<T> {
            const ENDIAN: EndianKind = <T as WriteEndian>::ENDIAN;
        }
    )+}
}

endian_generic_impl!(Option Vec PhantomData);

macro_rules! endian_tuple_impl {
    ($type1:ident $(, $types:ident)*) => {
        #[allow(non_camel_case_types)]
        impl<$type1: ReadEndian, $($types: ReadEndian),*> ReadEndian for ($type1, $($types),*) {
            const ENDIAN: EndianKind = EndianKind::Mixed;
        }

        #[allow(non_camel_case_types)]
        impl<$type1: WriteEndian, $($types: WriteEndian),*> WriteEndian for ($type1, $($types),*) {
            const ENDIAN: EndianKind = EndianKind::Mixed;
        }

        endian_tuple_impl!($($types),*);
    };

    () => {};
}

endian_tuple_impl!(
    b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19, b20, b21,
    b22, b23, b24, b25, b26, b27, b28, b29, b30, b31, b32
);
