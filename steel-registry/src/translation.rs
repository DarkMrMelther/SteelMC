use rustc_hash::FxHashMap;
use text_components::{build::TranslationsRegistry, translation::TranslationToken};

pub type TranslationRef = &'static [TranslationToken<'static>];

/// TODO: Add capabilities for other languages
pub struct TranslationRegistry {
    translations_by_key: FxHashMap<&'static str, TranslationRef>,
    allows_registering: bool,
}

impl TranslationRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            translations_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }

    pub fn get(&self, key: &str) -> Option<TranslationRef> {
        self.translations_by_key.get(key).map(|t| *t)
    }

    pub fn freeze(&mut self) {
        self.allows_registering = false;
    }
}

impl TranslationsRegistry for TranslationRegistry {
    fn register(
        &mut self,
        _locale: &'static str,
        key: &'static str,
        translation: &'static [TranslationToken<'static>],
    ) {
        if !self.allows_registering {
            return;
        };
        self.translations_by_key.insert(key, translation);
    }
}
