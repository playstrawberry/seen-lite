//! The script's texts: `lang/{en,fr,es,tr}.lang`, embedded and checked for the same keys while it compiles.
use crate::model::Part;
use sp_lang::{Catalog, Language};

static LANG: Catalog = sp_lang::catalog!("seen-lite");

/// The text of `key` in the language of `locale` (English for any other), with `{0}`, `{1}` filled from `args`.
pub fn text(locale: &str, key: &str, args: &[&str]) -> String {
    let raw = LANG
        .get_or_en(Language::of_locale(locale), key)
        .unwrap_or_default();
    sp_lang::render(raw, args, "")
}

/// A time span in words, in the language of `locale`: "2 days and 5 hours".
pub fn span(locale: &str, parts: &[Part]) -> String {
    let words: Vec<String> = parts
        .iter()
        .map(|(key, count)| text(locale, key, &[&count.to_string()]))
        .collect();
    match words.as_slice() {
        [one] => one.clone(),
        [first, second] => text(locale, "pair", &[first, second]),
        _ => words.join(" "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;

    #[test]
    fn every_key_the_script_names_has_a_text_in_every_language() {
        let own = [
            "description",
            "permission_use",
            "command_seen",
            "no_permission",
            "bad_name",
            "online",
            "offline",
            "never",
            "moment",
            "minute",
            "minutes",
            "hour",
            "hours",
            "day",
            "days",
            "pair",
        ];
        for key in own {
            for language in Language::ALL {
                assert!(
                    LANG.get(language, key).is_some_and(|t| !t.is_empty()),
                    "{key} in {}",
                    language.code()
                );
            }
        }
        assert_eq!(LANG.len(), own.len(), "a key nobody names");
    }

    #[test]
    fn a_time_reads_in_the_readers_language() {
        let parts = model::span(2 * 86_400_000 + 5 * 3_600_000);
        assert_eq!(span("en_us", &parts), "2 days and 5 hours");
        assert_eq!(span("fr_fr", &parts), "2 jours et 5 heures");
        assert_eq!(span("es_es", &parts), "2 días y 5 horas");
        assert_eq!(span("tr_tr", &parts), "2 gün 5 saat");
        assert_eq!(span("en_us", &model::span(60_000)), "1 minute");
        assert_eq!(span("de_de", &model::span(0)), "less than a minute");
    }

    #[test]
    fn every_answer_names_the_player_and_the_time_in_every_language() {
        for locale in ["en_us", "fr_fr", "es_es", "tr_tr"] {
            for key in ["online", "offline"] {
                let line = text(locale, key, &["Steve", "3 x"]);
                assert!(line.contains("Steve") && line.contains("3 x"), "{line}");
            }
            assert!(text(locale, "never", &["Steve"]).contains("Steve"));
        }
    }
}
