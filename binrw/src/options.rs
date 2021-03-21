use super::Endian;

/// Runtime-configured options for reading the type using [`BinRead`](crate::binread::BinRead)
#[non_exhaustive]
#[derive(Default, Clone, Copy)]
pub struct ReadOptions {
    pub endian: Endian,
    pub count: Option<usize>,
    pub offset: u64,
}
