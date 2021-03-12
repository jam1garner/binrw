macro_rules! define_keywords {
    ($($keyword:ident),+ $(,)?) => {
        $(
            syn::custom_keyword!($keyword);
        )*
    }
}

define_keywords! {
    align_after,
    align_before,
    args,
    args_tuple,
    assert,
    big,
    binread,
    br,
    calc,
    count,
    default,
    deref_now,
    ignore,
    import,
    import_tuple,
    is_big,
    is_little,
    little,
    magic,
    map,
    offset,
    offset_after,
    pad_after,
    pad_before,
    pad_size_to,
    parse_with,
    postprocess_now,
    pre_assert,
    repr,
    restore_position,
    return_all_errors,
    return_unexpected_error,
    seek_before,
    temp,
    try_map,
}
