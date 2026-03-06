use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn tr<'a>(&self, ru: &'a str, en: &'a str) -> Cow<'a, str> {
        match self.net.ui.language {
            Language::Ru => Cow::Borrowed(ru),
            Language::En => Cow::Borrowed(en),
        }
    }
}
