macro_rules! kws {
    ($($idents:ident),+ $(,)?) => {
        $(
            syn::custom_keyword!($idents);
        )*
    }
}

kws!{
    // attribute description
    br,
    binread,

    // all level
    big,        // big
    little,     // little
    magic,      // magic = [lit]
    assert,     // assert(expr)
    pre_assert,     // pre_assert(expr)

    // top-level
    import,     // import(expr, ..)
    import_tuple, // import(expr)
    repr,
    return_all_errors,
    return_unexpected_error,

    // field-level
    map,        // map = [func]
    parse_with, // parse_with = [func]
    calc,       // calc = [expr]
    count,      // count = [expr]
    is_little,  // is_little = [expr]
    is_big,     // is_big = [expr]
    args,       // args(expr, ..)
    args_tuple, // args_tuple = [expr]
    default,    // default
    ignore,     // ignore
    deref_now,  // deref_now
    restore_position,// restore_position
    postprocess_now, // postprocess_now
    offset,     // offset(expr)
    offset_after,     // offset(expr)
    /*if,*/     // if(expr)
    temp,

    pad_before,   // pad_before(expr)
    pad_after,    // pad_after(expr)
    align_before, // align_before(expr)
    align_after,  // align_after(expr)
    seek_before,  // seek_before(expr)
    pad_size_to,  // pad_size_to(expr)
}
