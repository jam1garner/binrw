/// A trait to allow for binrw types to provide a means of accessing their magic
pub trait HasMagic {
    type MagicType;
    const MAGIC: Self::MagicType;
}
