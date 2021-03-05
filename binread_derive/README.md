# binread_derive

## Quick start for adding a new directive to `BinRead`

In all cases, look to existing directives to follow established code and test
conventions.

1. Add a keyword for the new directive in `parser::keywords`.
2. Define the meta-type of the new directive in `parser::attrs`. If you need a
   new meta-type, add it to `parser::meta_types` along with tests.
3. If the new directive needs a special final type (e.g. `CondEndian`), add
   that to a new `parser::types` module and export it from `parser::types`. New
   types must ultimately implement `parser::TrySet`, but can sometimes do so
   more simply (using trait generic impls) by implementing `From` or `TryFrom`
   instead.
4. Add the new directive as a field to the relevant structs in
   `parser::top_level_attrs` and `parser::field_level_attrs`.
5. If the new directive combines with other directives in ways that may be
   invalid, and the relationship cannot be expressed using an enum type
   (e.g. `ReadMode`), add validation in either `FromInput::push_field` (if the
   validation can occur immediately after the field is constructed) or
   `FromInput::validate` (if it can only be validated after the entire struct
   has been parsed).
6. Use the new fields to emit code in the appropriate places in
   `codegen::read_options`.
7. Add new integration tests in the `binread` crate’s `tests` directory.
8. If the new directive generates new errors (e.g. from validation), add unit
   tests to validate those code paths in `parser::tests` (in `mod.rs`) and add
   identical trybuild tests to the `binread` crate’s `tests/ui` directory. (A
   nightly compiler is required to run the trybuild tests; see the comment in
   `binread::tests::ui` for more detail.)
