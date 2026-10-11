//! Guards the two invariants that the localization layer assumes but does not
//! enforce at runtime.
//!
//! `Localization::init` builds one bundle per available locale and silently
//! drops any locale whose FTL fails to parse (`if let Ok(res) = ...`). A single
//! syntax error therefore degrades a whole language to raw key names with no
//! warning, so it is checked here instead.
//!
//! `get` falls back from the current locale to `en` and, failing that, returns
//! the key verbatim. A message that exists in only some catalogs is invisible
//! in the locales that lack it, so key parity across catalogs is checked here
//! too.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// Locales that must stay in lockstep (see AGENTS.md, "Internationalization").
const LOCALES: &[&str] = &["en", "de", "ja"];

fn locale_path(locale: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../resources/locale")
        .join(locale)
        .join("main.ftl")
}

fn locale_source(locale: &str) -> String {
    let path = locale_path(locale);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()))
}

/// Message ids declared at the top level of an FTL file.
///
/// Every value in these catalogs is written inline on its message's line, so a
/// line is a declaration exactly when it starts in column 0, is not a comment,
/// and contains `=`. Indented attribute and continuation lines are excluded by
/// the column-0 requirement.
fn message_ids(locale: &str) -> BTreeSet<String> {
    locale_source(locale)
        .lines()
        .filter(|line| !line.starts_with([' ', '\t', '#']))
        .filter_map(|line| line.split_once('=').map(|(key, _)| key.trim().to_string()))
        .filter(|key| !key.is_empty())
        .collect()
}

#[test]
fn every_locale_catalog_parses() {
    for locale in LOCALES {
        let source = locale_source(locale);
        if let Err((_, errors)) = fluent::FluentResource::try_new(source) {
            // Report only the error kinds and positions; the Err payload also
            // carries the whole resource and its AST, which is far too much to
            // print and buries the useful part.
            let detail = errors
                .iter()
                .map(|e| format!("{:?} at byte {}", e.kind, e.pos.start))
                .collect::<Vec<_>>()
                .join("; ");
            panic!(
                "{locale}/main.ftl failed to parse, so `Localization::init` would drop the whole \
                 locale bundle and every string in it would render as its raw key: {detail}"
            );
        }
    }
}

#[test]
fn all_locales_define_the_same_messages() {
    let english = message_ids("en");
    assert!(
        !english.is_empty(),
        "parsed no message ids from en/main.ftl; the extraction above is out of date with the file"
    );

    let mut problems = Vec::new();
    for locale in LOCALES.iter().filter(|l| **l != "en") {
        let other = message_ids(locale);
        for missing in english.difference(&other) {
            problems.push(format!("{locale}/main.ftl is missing `{missing}`"));
        }
        for extra in other.difference(&english) {
            problems.push(format!(
                "{locale}/main.ftl defines `{extra}`, absent from en/main.ftl"
            ));
        }
    }

    assert!(
        problems.is_empty(),
        "locale catalogs have drifted; a message present in only some catalogs renders as its \
         raw key in the locales missing it:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn no_catalog_leaves_an_empty_value() {
    // `message_ids` only proves a key exists; an empty value would render as an
    // empty label rather than falling back to anything useful.
    let mut problems = Vec::new();
    for locale in LOCALES {
        for line in locale_source(locale).lines() {
            if line.starts_with([' ', '\t', '#']) {
                continue;
            }
            if let Some((key, value)) = line.split_once('=')
                && !key.trim().is_empty()
                && value.trim().is_empty()
            {
                problems.push(format!("{locale}/main.ftl: `{}` has no value", key.trim()));
            }
        }
    }

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
