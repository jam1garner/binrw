use super::*;

/// Runtime-configured options for reading the type using [`BinRead`](BinRead)
#[non_exhaustive]
#[derive(Default, Clone, Copy)]
pub struct ReadOptions {
    pub endian: Endian,
    pub count: Option<usize>,
    pub offset: u64,

    #[cfg(feature = "debug_template")]
    pub dont_output_to_template: bool,
    #[cfg(feature = "debug_template")]
    pub variable_name: Option<&'static str>,
}
