use binrw::BinrwNamedArgs;

#[derive(BinrwNamedArgs)]
union Foo {
    a: i32,
}

fn main() {}
