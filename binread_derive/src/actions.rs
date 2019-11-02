use super::attributes::AttrSetting;
use crate::binwrite_endian::Endian;
use proc_macro2::TokenStream;
use std::num::NonZeroUsize;

pub enum Action {
    Default,
    CutomerWriter(TokenStream),
}

#[derive(Default)]
pub struct GenOptions {
    pub pad_before: Option<NonZeroUsize>,
    pub pad_after: Option<NonZeroUsize>,
    pub align_before: Option<NonZeroUsize>,
    pub align_after: Option<NonZeroUsize>,
    pub preprocessor: Option<TokenStream>,
}

#[derive(Default)]
pub struct OptionalWriterOption {
    pub endian: Option<Endian>
}

pub struct WriteInstructions(pub Action, pub OptionalWriterOption, pub GenOptions);

impl WriteInstructions {
    pub fn try_from(settings: &Vec<AttrSetting>) -> Option<WriteInstructions> {
        let mut action: Action = Action::Default;
        let mut writer_option = OptionalWriterOption::default();
        let mut gen_options = GenOptions::default();
        for setting in settings.iter() {
            match setting {
                AttrSetting::Endian(endian) => {
                    writer_option.endian = Some(*endian);
                }
                AttrSetting::With(writer_func) => {
                    action = Action::CutomerWriter(writer_func.clone());
                }
                AttrSetting::Preprocessor(preprocessor) => {
                    gen_options.preprocessor = Some(preprocessor.clone());
                }
                AttrSetting::AlignBefore(pad) => {
                    gen_options.align_before = NonZeroUsize::new(*pad);
                }
                AttrSetting::AlignAfter(pad) => {
                    gen_options.align_after = NonZeroUsize::new(*pad);
                }
                AttrSetting::PadBefore(pad) => {
                    gen_options.pad_before = NonZeroUsize::new(*pad);
                }
                AttrSetting::PadAfter(pad) => {
                    gen_options.pad_after = NonZeroUsize::new(*pad);
                }
                AttrSetting::Ignore => {
                    None?
                }
            }
        }
        
        Some(WriteInstructions(action, writer_option, gen_options))
    }
}
