use std::collections::HashMap;

use fluent::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

mod locales_codegen {
    include!(concat!(env!("OUT_DIR"), "/locales_codegen.rs"));
}
pub use locales_codegen::locale_display_name;

pub const AVAILABLE_LOCALES: &[&str] = locales_codegen::AVAILABLE_LOCALES;

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
        let bundle = self.bundles.get(&self.current_lang)
            .or_else(|| self.bundles.get(&"en".parse().unwrap()));
        if let Some(bundle) = bundle {
            let mut errors = Vec::new();
            if let Some(pattern) = bundle.get_message(key) {
                if let Some(value) = pattern.value() {
                    let mut args_map = fluent::FluentArgs::new();
                    for (k, v) in args {
                        args_map.set(*k, *v);
                    }
                    return bundle
                        .format_pattern(value, Some(&args_map), &mut errors)
                        .into_owned();
                }
            }
        }
        key.to_string()
    }
}
