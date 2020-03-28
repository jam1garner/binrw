use super::*;

macro_rules! test_tla {
    ($name:ident, $str:literal) => {
        #[test]
        fn $name() {
            let tokens: TokenStream2 = ($str).parse().unwrap();
            let _: TopLevelAttr = syn::parse2(tokens).unwrap();
        }
    }
}
macro_rules! test_fla {
    ($name:ident, $str:literal) => {
        #[test]
        fn $name() {
            let tokens: TokenStream2 = ($str).parse().unwrap();
            let _: FieldLevelAttr = syn::parse2(tokens).unwrap();
        }
    }
}

macro_rules! parse_ty {
    ($name:ident, $str:literal, $ty:ty) => {
        #[test]
        fn $name() {
            let tokens: TokenStream2 = ($str).parse().unwrap();
            let _: $ty = syn::parse2(tokens).unwrap();
        }
    }
}

macro_rules! parse_ty_print {
    ($name:ident, $str:literal, $ty:ty) => {
        #[test]
        fn $name() {
            let tokens: TokenStream2 = ($str).parse().unwrap();
            let val: $ty = syn::parse2(tokens).unwrap();
            dbg!(val);
        }
    }
}

macro_rules! parse_ty_fail {
    ($name:ident, $str:literal, $ty:ty) => {
        #[test]
        #[should_panic]
        fn $name() {
            let tokens: TokenStream2 = ($str).parse().unwrap();
            let _: $ty = syn::parse2(tokens).unwrap();
        }
    }
}

test_tla!(parse_big, "big");
test_tla!(parse_magic, "magic = 3u8");
test_tla!(parse_magic_paren, "magic(2u16)");
test_tla!(parse_import, "import(x: u32, y: &[f32])");

test_fla!(fla_little, "little");
test_fla!(fla_magic, "magic = b\"TEST\"");
test_fla!(fla_if, "if(x == 1)");
test_fla!(fla_map, "map = |val: u32| val.to_string()");
test_fla!(fla_seek_before, "seek_before(SeekFrom::Current(4))");
test_fla!(fla_parse_with, "parse_with = read_offsets");
test_fla!(fla_ignore, "ignore");
test_fla!(fla_assert, "assert(
    offsets.1 - offsets.0 == 0x10,
    BadDifferenceError(offsets.1 - offsets.0)
)");
test_fla!(fla_count, "count = extra_entry_count + 1");
test_fla!(fla_args, "args(x, (y, z), 3 + 4)");
test_fla!(fla_default, "default");

parse_ty!(meta_bool, "little", kw::little);
parse_ty!(meta_lit, "magic = 3u8", MetaLit<kw::magic>);
parse_ty!(meta_byte_lit, "magic = b\"TEST\"", MetaLit<kw::magic>);
parse_ty!(meta_str_lit, "magic = \"string\"", MetaLit<kw::magic>);
parse_ty!(meta_func_closure, "map = |x| x + 1", MetaFunc<kw::map>);
parse_ty!(meta_func_path, "map = ToString::to_string", MetaFunc<kw::map>);

parse_ty_fail!(meta_lit_panic, "= 3u8", MetaLit<kw::magic>);
parse_ty_fail!(meta_lit_panic2, "test = 3u8", MetaLit<kw::magic>);
