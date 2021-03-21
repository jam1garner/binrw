use crate::Endian;

/// Runtime-configured options for reading the type using [`BinRead`](BinRead)
#[non_exhaustive]
#[derive(Default, Clone, Copy)]
pub struct ReadOptions {
    pub endian: Endian,
    pub count: Option<usize>,
    pub offset: u64,
}

#[derive(Default, Clone, Copy)]
pub struct WriteOptions {
    endian: Endian,
}

impl WriteOptions {
    pub fn new() -> Self {
        Self {
            endian: Endian::Native
        }
    }

    pub fn with_endian(self, endian: Endian) -> Self {
        Self {
            endian,
            ..self
        }
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }
}
