use std::collections::HashMap;

use fluent::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

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

        let ftl = r#"
move-action-message = Move {$file_name} to {$directory}
delete-action-message = Delete {$file_name}
rename-action-message = Rename {$old_file_name} to {$new_file_name}
could-not-act-error = Could not execute action "{$act_message}": {$error_message}
        "#;

        if let Ok(res) = FluentResource::try_new(ftl.to_string()) {
            let mut bundle = FluentBundle::new(vec![langid.clone()]);
            bundle.add_resource(res).ok();
            bundles.insert(langid.clone(), bundle);
        }

        Self {
            bundles,
            current_lang: langid,
        }
    }

    pub fn get(&self, key: &str, args: &[(&str, &str)]) -> String {
        if let Some(bundle) = self.bundles.get(&self.current_lang) {
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
