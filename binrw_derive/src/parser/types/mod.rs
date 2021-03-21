mod assert;
mod cond_endian;
mod condition;
mod enum_error_mode;
mod imports;
mod magic;
mod map;
mod passed_args;
mod read_mode;
mod spanned_value;

pub(crate) use assert::{Assert, Error as AssertionError};
pub(crate) use cond_endian::{CondEndian, Endian};
pub(crate) use condition::Condition;
pub(crate) use enum_error_mode::EnumErrorMode;
pub(crate) use imports::Imports;
pub(crate) use magic::Magic;
pub(crate) use map::Map;
pub(crate) use passed_args::PassedArgs;
pub(crate) use read_mode::ReadMode;
pub(crate) use spanned_value::SpannedValue;

fn assert_all_args_consumed<Iter, IterItem>(
    args: Iter,
    default_span: proc_macro2::Span,
) -> syn::Result<()>
where
    IterItem: syn::spanned::Spanned,
    Iter: Iterator<Item = IterItem>,
{
    let mut extra_span = None::<proc_macro2::Span>;
    for extra_arg in args {
        let arg_span = extra_arg.span();
        if let Some(span) = extra_span {
            // This join will fail if the `proc_macro_span` feature is
            // unavailable. Falling back to the `ident` span is better than
            // doing nothing.
            if let Some(new_span) = span.join(arg_span) {
                extra_span = Some(new_span);
            } else {
                extra_span = Some(default_span);
                break;
            }
        } else {
            extra_span = Some(arg_span);
        }
    }

    extra_span.map_or(Ok(()), |span| {
        Err(syn::Error::new(span, "too many arguments"))
    })
}
