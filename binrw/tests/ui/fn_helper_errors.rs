use binrw::{parser, writer, BinResult};

#[parser(reader:)]
fn fn_helper_invalid_option_value() -> BinResult<()> {
    Ok(())
}

#[parser(reader = invalid)]
fn fn_helper_invalid_option_token() -> BinResult<()> {
    Ok(())
}

#[parser(invalid)]
fn fn_helper_invalid_reader() -> BinResult<()> {
    Ok(())
}

#[writer(invalid)]
fn fn_helper_invalid_writer() -> BinResult<()> {
    Ok(())
}

#[parser(reader, reader)]
fn fn_helper_conflicting_reader() -> BinResult<()> {
    Ok(())
}

#[writer(writer, writer)]
fn fn_helper_conflicting_writer() -> BinResult<()> {
    Ok(())
}

#[parser(endian, endian)]
fn fn_helper_conflicting_endian() -> BinResult<()> {
    Ok(())
}

struct InvalidSelf;
impl InvalidSelf {
    #[parser]
    fn fn_helper_invalid_self(&self) -> BinResult<()> {
        Ok(())
    }
}

#[writer]
fn fn_helper_missing_object() -> BinResult<()> {
    Ok(())
}

#[parser]
fn fn_helper_missing_args_reader(...) -> BinResult<()> {
    Ok(())
}

#[parser]
fn fn_helper_extra_args_reader(arg0: (), arg1: (), ...) -> BinResult<()> {
    Ok(())
}

#[writer]
fn fn_helper_extra_args_writer(arg0: &(), arg1: (), arg2: (), ...) -> BinResult<()> {
    Ok(())
}

#[writer]
fn fn_helper_missing_args_writer(obj: &(), ...) -> BinResult<()> {
    Ok(())
}

fn main() {}
