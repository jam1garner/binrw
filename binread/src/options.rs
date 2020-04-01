use super::*;

/// Runtime-configured options for reading the type using [`BinRead`](BinRead)
#[non_exhaustive]
#[derive(Default, Clone, Copy)]
pub struct ReadOptions {
    pub endian: Endian,
    pub count: Option<usize>,
    
    #[cfg(feature = "debug_template")]
    pub dont_output_to_template: bool,
    #[cfg(feature = "debug_template")]
    pub variable_name: Option<&'static str>,
}

/// Runtime-configured options for what to do with the type after the parent has been parsed
#[non_exhaustive]
#[derive(Default, Clone, Copy)]
pub struct AfterParseOptions {
    pub offset: u64
}

