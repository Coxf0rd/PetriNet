use super::*;
use encoding_rs::WINDOWS_1251;

impl PetriApp {
    pub(in crate::ui::app) fn tr<'a>(&self, ru: &'a str, en: &'a str) -> Cow<'a, str> {
        match self.net.ui.language {
            Language::Ru => Self::restore_ru_mojibake(ru)
                .map(Cow::Owned)
                .unwrap_or_else(|| Cow::Borrowed(ru)),
            Language::En => Cow::Borrowed(en),
        }
    }

    fn restore_ru_mojibake(input: &str) -> Option<String> {
        if !Self::looks_like_mojibake(input) {
            return None;
        }

        let (bytes, _, had_encode_errors) = WINDOWS_1251.encode(input);
        if had_encode_errors {
            return None;
        }

        let decoded = String::from_utf8(bytes.into_owned()).ok()?;
        if !Self::contains_cyrillic(&decoded) {
            return None;
        }

        let src_score = Self::mojibake_score(input);
        let dst_score = Self::mojibake_score(&decoded);
        (dst_score < src_score).then_some(decoded)
    }

    fn contains_cyrillic(text: &str) -> bool {
        text.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c))
    }

    fn looks_like_mojibake(text: &str) -> bool {
        text.contains('Р') || text.contains('С') || text.contains("вЂ") || text.contains('Џ')
    }

    fn mojibake_score(text: &str) -> usize {
        let mut score = 0usize;
        for pattern in ["Р", "С", "вЂ", "Ѓ", "Џ", "Ў"] {
            score = score.saturating_add(text.matches(pattern).count());
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restores_common_utf8_cp1251_mojibake() {
        let src = "РњР°СЂРєРѕРІСЃРєР°СЏ РјРѕРґРµР»СЊ";
        let restored = PetriApp::restore_ru_mojibake(src).expect("restored");
        assert_eq!(restored, "Марковская модель");
    }

    #[test]
    fn keeps_valid_russian_unchanged() {
        let src = "Марковская модель";
        assert!(PetriApp::restore_ru_mojibake(src).is_none());
    }
}
