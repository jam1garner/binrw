#[doc(hidden)]
pub struct Satisfied;

#[doc(hidden)]
pub struct Optional;

#[doc(hidden)]
pub struct Needed;

// TODO: seal?
/// Indicates that a requirement for a typed builder has been met, either by
/// the user providing one, or by a default being given.
#[doc(hidden)]
pub trait SatisfiedOrOptional {}

impl SatisfiedOrOptional for Satisfied {}
impl SatisfiedOrOptional for Optional {}
