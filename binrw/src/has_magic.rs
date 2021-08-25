pub trait HasMagic {
    type MagicType;
    const MAGIC: Self::MagicType;
}
