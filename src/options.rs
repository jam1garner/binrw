use super::*;

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

pub type Imports = Vec<Option<Box<dyn Any>>>;

#[non_exhaustive]
#[derive(Default, Clone, Copy)]
pub struct AfterParseOptions {
    pub offset: u64
}

