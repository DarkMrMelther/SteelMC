use rustc_hash::FxHashMap;
use text_components::{
    TextComponent,
    custom::{CustomContent, CustomContentExt},
};

pub type TextContentRef = &'static (dyn CustomContent<'static, ()> + Send + Sync);

pub struct TextContentRegistry {
    text_contents_by_key: FxHashMap<&'static str, Vec<TextContentRef>>,
    allows_registering: bool,
}

impl TextContentRegistry {
    #[must_use]
    pub fn new() -> Self {
        let mut new = Self {
            text_contents_by_key: FxHashMap::default(),
            allows_registering: true,
        };
        new.register(&HelloWorldContent);
        new
    }

    pub fn get(&self, key: &str) -> Option<TextContentRef> {
        let (namespace, path) = key.split_once(':').unwrap_or(("", key));
        self.text_contents_by_key.get(path).map(|vec| {
            if vec.len() == 1 || namespace.is_empty() {
                vec[0]
            } else {
                vec.iter()
                    .find(|content| content.namespace() == namespace)
                    .map_or(vec[0], |cc| *cc)
            }
        })
    }

    pub fn register(&mut self, content: &'static (impl CustomContent<'static, ()> + Send + Sync)) {
        if !self.allows_registering {
            return;
        };
        let Some(vec) = self.text_contents_by_key.get_mut(content.id()) else {
            self.text_contents_by_key
                .insert(content.id(), vec![content]);
            return;
        };
        for (i, inner_content) in vec.iter().enumerate() {
            if inner_content.namespace() == content.namespace() {
                vec.swap_remove(i);
                vec.push(content);
                let last = vec.len() - 1;
                vec.swap(i, last);
                return;
            }
        }
        vec.push(content);
    }

    pub fn freeze(&mut self) {
        self.allows_registering = false;
    }
}

struct HelloWorldContent;
impl CustomContentExt<'static> for HelloWorldContent {
    fn namespace(&self) -> &'static str {
        "test"
    }

    fn id(&self) -> &'static str {
        "hello"
    }

    fn payload(&self) -> text_components::custom::Payload {
        text_components::custom::Payload::Empty
    }
}
impl CustomContent<'static, ()> for HelloWorldContent {
    fn resolve<'a>(
        &self,
        _resolutor: &dyn text_components::resolving::TextResolutor<'a>,
        _context: (),
        _payload: text_components::custom::Payload,
    ) -> text_components::RawTextComponent<'a> {
        TextComponent::const_plain("Hello World from Custom Content!")
    }
}
