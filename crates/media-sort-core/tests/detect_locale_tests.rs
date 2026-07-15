use std::sync::Mutex;

static LOCALE_TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_detect_locale_exact_match() {
    let _guard = LOCALE_TEST_MUTEX.lock().unwrap();
    let old_lang = std::env::var("LANG").ok();
    let old_lc_all = std::env::var("LC_ALL").ok();
    unsafe {
        std::env::remove_var("LC_ALL");
        std::env::set_var("LANG", "en_US.UTF-8");
    }
    let locale = media_sort_core::l10n::detect_locale();
    assert_eq!(locale, "en");
    if let Some(ref val) = old_lang {
        unsafe {
            std::env::set_var("LANG", val);
        }
    } else {
        unsafe {
            std::env::remove_var("LANG");
        }
    }
    if let Some(ref val) = old_lc_all {
        unsafe {
            std::env::set_var("LC_ALL", val);
        }
    } else {
        unsafe {
            std::env::remove_var("LC_ALL");
        }
    }
}

#[test]
fn test_detect_locale_prefix_match() {
    let _guard = LOCALE_TEST_MUTEX.lock().unwrap();
    let old_lang = std::env::var("LANG").ok();
    let old_lc_all = std::env::var("LC_ALL").ok();
    unsafe {
        std::env::remove_var("LC_ALL");
        std::env::set_var("LANG", "de_AT.UTF-8");
    }
    let locale = media_sort_core::l10n::detect_locale();
    assert_eq!(locale, "de");
    if let Some(ref val) = old_lang {
        unsafe {
            std::env::set_var("LANG", val);
        }
    } else {
        unsafe {
            std::env::remove_var("LANG");
        }
    }
    if let Some(ref val) = old_lc_all {
        unsafe {
            std::env::set_var("LC_ALL", val);
        }
    } else {
        unsafe {
            std::env::remove_var("LC_ALL");
        }
    }
}

#[test]
fn test_detect_locale_ja_exact() {
    let _guard = LOCALE_TEST_MUTEX.lock().unwrap();
    let old_lang = std::env::var("LANG").ok();
    let old_lc_all = std::env::var("LC_ALL").ok();
    unsafe {
        std::env::remove_var("LC_ALL");
        std::env::set_var("LANG", "ja_JP.UTF-8");
    }
    let locale = media_sort_core::l10n::detect_locale();
    assert_eq!(locale, "ja");
    if let Some(ref val) = old_lang {
        unsafe {
            std::env::set_var("LANG", val);
        }
    } else {
        unsafe {
            std::env::remove_var("LANG");
        }
    }
    if let Some(ref val) = old_lc_all {
        unsafe {
            std::env::set_var("LC_ALL", val);
        }
    } else {
        unsafe {
            std::env::remove_var("LC_ALL");
        }
    }
}

#[test]
fn test_detect_locale_no_match_fallback() {
    let _guard = LOCALE_TEST_MUTEX.lock().unwrap();
    let old_lang = std::env::var("LANG").ok();
    let old_lc_all = std::env::var("LC_ALL").ok();
    unsafe {
        std::env::remove_var("LC_ALL");
        std::env::set_var("LANG", "fr_FR.UTF-8");
    }
    let locale = media_sort_core::l10n::detect_locale();
    assert_eq!(locale, "en");
    if let Some(ref val) = old_lang {
        unsafe {
            std::env::set_var("LANG", val);
        }
    } else {
        unsafe {
            std::env::remove_var("LANG");
        }
    }
    if let Some(ref val) = old_lc_all {
        unsafe {
            std::env::set_var("LC_ALL", val);
        }
    } else {
        unsafe {
            std::env::remove_var("LC_ALL");
        }
    }
}

#[test]
fn test_detect_locale_lc_all_priority() {
    let _guard = LOCALE_TEST_MUTEX.lock().unwrap();
    let old_lc_all = std::env::var("LC_ALL").ok();
    let old_lang = std::env::var("LANG").ok();
    unsafe {
        std::env::set_var("LC_ALL", "de_DE.UTF-8");
        std::env::set_var("LANG", "en_US.UTF-8");
    }
    let locale = media_sort_core::l10n::detect_locale();
    assert_eq!(locale, "de");
    if let Some(ref val) = old_lc_all {
        unsafe {
            std::env::set_var("LC_ALL", val);
        }
    } else {
        unsafe {
            std::env::remove_var("LC_ALL");
        }
    }
    if let Some(ref val) = old_lang {
        unsafe {
            std::env::set_var("LANG", val);
        }
    } else {
        unsafe {
            std::env::remove_var("LANG");
        }
    }
}
