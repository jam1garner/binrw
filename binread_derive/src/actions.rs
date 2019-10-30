use super::attributes::AttrSetting;
use binwrite::WriterOption;
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
    pub preprocessor: Option<TokenStream>,
}

pub struct WriteInstructions(pub Action, pub WriterOption, pub GenOptions);

impl From<&Vec<AttrSetting>> for WriteInstructions {
    fn from(settings: &Vec<AttrSetting>) -> WriteInstructions {
        let mut action: Action = Action::Default;
        let mut writer_option = WriterOption::default();
        let mut gen_options = GenOptions::default();
        for setting in settings.iter() {
            match setting {
                AttrSetting::Endian(endian) => {
                    writer_option.endian = *endian;
                }
                AttrSetting::With(writer_func) => {
                    action = Action::CutomerWriter(writer_func.clone());
                }
                AttrSetting::PadBefore(pad) => {
                    gen_options.pad_before = NonZeroUsize::new(*pad);
                }
                AttrSetting::PadAfter(pad) => {
                    gen_options.pad_after = NonZeroUsize::new(*pad);
                }
            }
        }
        WriteInstructions(action, writer_option, gen_options)
    }
}
