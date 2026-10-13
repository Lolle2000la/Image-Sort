use std::collections::HashMap;

use fluent::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

mod locales_codegen {
    include!(concat!(env!("OUT_DIR"), "/locales_codegen.rs"));
}
pub use locales_codegen::locale_display_name;

pub const AVAILABLE_LOCALES: &[&str] = locales_codegen::AVAILABLE_LOCALES;

pub fn detect_locale() -> &'static str {
    for var in &["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            let val_lower = val.to_ascii_lowercase();
            let val_norm = val_lower.replace('_', "-");

            // Pass 1: exact match
            if let Some(&lang) = AVAILABLE_LOCALES
                .iter()
                .find(|&&lang| val_norm == lang.to_ascii_lowercase().replace('_', "-"))
            {
                return lang;
            }

            // Pass 2: longest prefix match
            if let Some(&lang) = AVAILABLE_LOCALES
                .iter()
                .filter(|&&lang| {
                    let lang_norm = lang.to_ascii_lowercase().replace('_', "-");
                    val_norm
                        .strip_prefix(&lang_norm)
                        .is_some_and(|rest| rest.starts_with(['-', '.', '@']))
                })
                .max_by_key(|&&lang| lang.len())
            {
                return lang;
            }
        }
    }
    "en"
}

pub struct Localization {
    bundles: HashMap<LanguageIdentifier, FluentBundle<FluentResource>>,
    current_lang: LanguageIdentifier,
}

impl Localization {
    pub fn init(default_lang: &str) -> Self {
        let langid: LanguageIdentifier = default_lang
            .parse()
            .unwrap_or_else(|_| "en".parse().unwrap());
        let mut bundles = HashMap::new();

        for &lang_code in AVAILABLE_LOCALES {
            let ftl_content = if lang_code == "en" {
                include_str!("../../../resources/locale/en/main.ftl")
            } else {
                locales_codegen::load_ftl(lang_code).unwrap()
            };

            let current_id: LanguageIdentifier = lang_code.parse().unwrap();
            if let Ok(res) = FluentResource::try_new(ftl_content.to_string()) {
                let mut bundle = FluentBundle::new(vec![current_id.clone()]);
                bundle.add_resource(res).ok();
                bundle.set_use_isolating(false);
                bundles.insert(current_id, bundle);
            }
        }

        Self {
            bundles,
            current_lang: langid,
        }
    }

    pub fn get(&self, key: &str, args: &[(&str, &str)]) -> String {
        let bundle = self
            .bundles
            .get(&self.current_lang)
            .or_else(|| self.bundles.get(&"en".parse().unwrap()));
        if let Some(bundle) = bundle {
            let mut errors = Vec::new();
            if let Some(pattern) = bundle.get_message(key)
                && let Some(value) = pattern.value()
            {
                let mut args_map = fluent::FluentArgs::new();
                for (k, v) in args {
                    args_map.set(*k, *v);
                }
                return bundle
                    .format_pattern(value, Some(&args_map), &mut errors)
                    .into_owned();
            }
        }
        key.to_string()
    }

    pub fn tr(&self, key: &str) -> String {
        self.get(key, &[])
    }

    pub fn set_locale(&mut self, lang: &str) {
        self.current_lang = lang.parse().unwrap_or_else(|_| "en".parse().unwrap());
    }

    pub fn locale(&self) -> String {
        self.current_lang.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::l10n::Localization;

    #[test]
    fn test_l10n_init() {
        let loc = Localization::init("en");
        assert!(
            !loc.get(
                "move-action-message",
                &[("file_name", "test.jpg"), ("directory", "/home")]
            )
            .is_empty()
        );
    }

    #[test]
    fn test_l10n_get_known_key() {
        let loc = Localization::init("en");
        let msg = loc.get(
            "move-action-message",
            &[("file_name", "photo.png"), ("directory", "/pics")],
        );
        assert!(!msg.is_empty());
        assert!(msg.contains("photo.png") || msg.contains("/pics"));
    }

    #[test]
    fn test_l10n_unknown_key_fallback() {
        let loc = Localization::init("en");
        let result = loc.get("nonexistent_key", &[]);
        assert_eq!(result, "nonexistent_key");
    }

    #[test]
    fn test_l10n_delete_message() {
        let loc = Localization::init("en");
        let msg = loc.get("delete-action-message", &[("file_name", "old_file.dat")]);
        assert!(!msg.is_empty());
        assert!(msg.contains("old_file.dat"));
    }

    #[test]
    fn test_l10n_rename_message() {
        let loc = Localization::init("en");
        let msg = loc.get(
            "rename-action-message",
            &[("old_file_name", "a.jpg"), ("new_file_name", "b.jpg")],
        );
        assert!(!msg.is_empty());
        assert!(msg.contains("a.jpg") || msg.contains("b.jpg"));
    }

    #[test]
    fn test_l10n_invalid_language_fallback() {
        let loc = Localization::init("invalid-lang-tag-!!!");
        let msg = loc.get(
            "move-action-message",
            &[("file_name", "test.txt"), ("directory", "/tmp")],
        );
        assert!(!msg.is_empty());
        assert!(msg.contains("Move"));
    }

    #[test]
    fn test_l10n_get_with_extra_args() {
        let loc = Localization::init("en");
        let msg = loc.get(
            "delete-action-message",
            &[("file_name", "test.txt"), ("extra_unused", "ignored")],
        );
        assert!(!msg.is_empty());
        assert!(msg.contains("test.txt"));
    }

    #[test]
    fn test_l10n_get_missing_args() {
        let loc = Localization::init("en");
        let msg = loc.get("move-action-message", &[]);
        assert!(!msg.is_empty());
    }
}
