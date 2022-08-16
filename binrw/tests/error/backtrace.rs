use binrw::BinRead;

#[derive(BinRead)]
struct InnerMostStruct {
    #[br(little)]
    len: u32,

    #[br(count = len)]
    _items: Vec<u32>,
}

#[allow(dead_code)]
#[derive(BinRead)]
enum MiddleEnum {
    OnlyOption {
        #[br(big)]
        #[br(assert(inner.len == 3))]
        inner: InnerMostStruct,
    },

    OtherOption(u32, u32),
}

#[derive(BinRead)]
struct MiddleStruct {
    #[br(little)]
    _middle: MiddleEnum,
}

#[derive(BinRead)]
pub struct OutermostStruct {
    #[br(little)]
    _middle: MiddleStruct,
}
