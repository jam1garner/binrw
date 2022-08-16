use binrw::BinRead;

#[allow(dead_code)]
#[derive(BinRead)]
struct InnerMostStruct {
    #[br(little)]
    len: u32,

    #[br(count = len, err_context("len = {}", len))]
    items: Vec<u32>,
}

#[derive(BinRead)]
struct MiddleStruct {
    #[br(little)]
    #[br(err_context("While parsing the innerest most struct"))]
    _inner: InnerMostStruct,
}

#[derive(BinRead)]
pub struct OutermostStruct {
    #[br(little)]
    _middle: MiddleStruct,
}
