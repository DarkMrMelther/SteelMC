//! Server utilities related to the text_components library

use steel_registry::{REGISTRY, translation::TranslationRef};
use text_components::{
    RawTextComponent,
    content::{Content, Resolvable},
    custom::CustomData,
    resolving::{ResolutionHelper, TextResolutor},
};

/// Provides global data for resolving content within a text component.
pub struct GlobalResolutionHelper;
impl<'a> ResolutionHelper<'a> for GlobalResolutionHelper {
    fn resolve_custom(
        &self,
        resolutor: &dyn TextResolutor<'a>,
        data: &CustomData<'a>,
    ) -> Option<text_components::RawTextComponent<'a>> {
        REGISTRY
            .text_contents
            .get(&data.id)
            .map(|cc| cc.resolve(resolutor, (), data.payload.clone()))
    }

    fn translate(&self, _locale: &str, key: &str) -> Option<TranslationRef> {
        REGISTRY.translations.get(key)
    }
}

/// A [`TextResolutor`] for the console
pub struct DisplayResolutor;
impl<'a> TextResolutor<'a> for DisplayResolutor {
    fn resolve_content(&self, resolvable: &Resolvable<'a>) -> RawTextComponent<'a> {
        RawTextComponent {
            content: Content::Resolvable(resolvable.clone()),
            ..Default::default()
        }
    }
}
