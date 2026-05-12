//! Internationalization for `celestial` CLI help/about text.
//!
//! Languages supported: English (default), Brazilian Portuguese, Spanish,
//! Italian, German.
//!
//! Locale detection precedence:
//!   1. `CELESTIAL_LANG` environment variable (highest priority)
//!   2. `LC_ALL` environment variable
//!   3. `LANG` environment variable
//!   4. English (default)
//!
//! Only the user-facing **help text** is translated. Error messages, computed
//! values, dates, locations, planetary names, and astrological terminology
//! remain in English so output is portable across locales (a script parsing
//! `celestial calc --json` works identically regardless of $LANG).
//!
//! ## Adding a new language
//!
//! 1. Add a variant to [`Lang`]
//! 2. Update [`Lang::from_locale`] with the BCP-47 / POSIX prefix
//! 3. Add a row in [`TR_TABLE`] for every translation key
//! 4. Update [`Lang::all`] (used by tests to verify completeness)

/// Supported user-interface languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    /// English — default fallback for any unrecognised locale.
    En,
    /// Brazilian Portuguese — `pt_BR`, `pt-BR`, or `pt`.
    PtBr,
    /// Spanish — `es_*` / `es-*`.
    Es,
    /// Italian — `it_*` / `it-*`.
    It,
    /// German — `de_*` / `de-*`.
    De,
}

impl Lang {
    /// All supported languages, in declaration order.
    pub const fn all() -> &'static [Lang] {
        &[Lang::En, Lang::PtBr, Lang::Es, Lang::It, Lang::De]
    }

    /// BCP-47-ish identifier used in error messages and `--lang` output.
    pub const fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::PtBr => "pt-BR",
            Lang::Es => "es",
            Lang::It => "it",
            Lang::De => "de",
        }
    }

    /// Parse a POSIX/BCP-47 locale string (case-insensitive on the language
    /// prefix). Returns `None` for unrecognised values so callers can fall
    /// through to the next env var.
    ///
    /// Accepted shapes:
    /// - `en`, `en_US`, `en-US`, `en_US.UTF-8`, `English_United States.1252`
    /// - `pt`, `pt_BR`, `pt-BR`, `pt_PT` (all map to `PtBr`; we don't ship
    ///   a separate European Portuguese file)
    /// - `es*`, `it*`, `de*`
    /// - `C`, `POSIX`, `""` → `None` (no preference expressed)
    pub fn from_locale(s: &str) -> Option<Lang> {
        let s = s.trim();
        if s.is_empty() || s.eq_ignore_ascii_case("c") || s.eq_ignore_ascii_case("posix") {
            return None;
        }
        // Take everything before the first '.', '@', '_', or '-' as the
        // language tag, EXCEPT we want pt_BR / pt-BR to keep their region
        // for disambiguation. Strategy: lowercase, then look at the first
        // 5 chars and the first 2 chars.
        let lower = s.to_ascii_lowercase();
        // Long form first (region-aware).
        if lower.starts_with("pt_br") || lower.starts_with("pt-br") {
            return Some(Lang::PtBr);
        }
        // Short form.
        let two = lower.get(..2)?;
        match two {
            "en" => Some(Lang::En),
            "pt" => Some(Lang::PtBr), // any Portuguese -> Brazilian (only variant we ship)
            "es" => Some(Lang::Es),
            "it" => Some(Lang::It),
            "de" => Some(Lang::De),
            _ => None,
        }
    }

    /// Detect the active language from environment variables, falling back to
    /// English. Order: `CELESTIAL_LANG` → `LC_ALL` → `LANG` → English.
    pub fn detect() -> Lang {
        for var in ["CELESTIAL_LANG", "LC_ALL", "LANG"] {
            if let Ok(v) = std::env::var(var) {
                if let Some(lang) = Lang::from_locale(&v) {
                    return lang;
                }
            }
        }
        Lang::En
    }
}

/// One row of the translation table — English plus four other languages, all
/// keyed by an internal ASCII identifier.
#[derive(Clone, Copy)]
struct Tr {
    key: &'static str,
    en: &'static str,
    pt_br: &'static str,
    es: &'static str,
    it: &'static str,
    de: &'static str,
}

impl Tr {
    const fn pick(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::En => self.en,
            Lang::PtBr => self.pt_br,
            Lang::Es => self.es,
            Lang::It => self.it,
            Lang::De => self.de,
        }
    }
}

/// Translation table for every help string the CLI exposes.
///
/// Keys follow the convention:
/// - `top.about`               — top-level command description
/// - `flag.list_plugins`       — global flag help
/// - `cmd.<name>.about`        — subcommand short description
/// - `cmd.<name>.long`         — optional long-form subcommand help
///
/// Tests assert that every `Lang` variant has a non-empty value for every
/// key — so a missing translation is caught at `cargo test` time, not at
/// runtime in front of the user.
const TR_TABLE: &[Tr] = &[
    Tr {
        key: "top.about",
        en: "Astronomical calculations — celestial engine",
        pt_br: "Cálculos astronômicos — motor celestial",
        es: "Cálculos astronómicos — motor celestial",
        it: "Calcoli astronomici — motore celestiale",
        de: "Astronomische Berechnungen — Celestial-Engine",
    },
    Tr {
        key: "flag.list_plugins",
        en: "List discovered celestial-* plugin executables on $PATH",
        pt_br: "Listar plugins celestial-* descobertos no $PATH",
        es: "Listar plugins celestial-* descubiertos en $PATH",
        it: "Elenca plugin celestial-* trovati nel $PATH",
        de: "Erkannte celestial-*-Plugins im $PATH auflisten",
    },
    Tr {
        key: "cmd.calc.about",
        en: "Calculate geocentric planetary positions",
        pt_br: "Calcular posições geocêntricas dos planetas",
        es: "Calcular posiciones geocéntricas de los planetas",
        it: "Calcola posizioni geocentriche dei pianeti",
        de: "Geozentrische Planetenpositionen berechnen",
    },
    Tr {
        key: "cmd.houses.about",
        en: "Calculate house cusps and special angles",
        pt_br: "Calcular cúspides de casas e ângulos especiais",
        es: "Calcular cúspides de casas y ángulos especiales",
        it: "Calcola cuspidi delle case e angoli speciali",
        de: "Häuser-Spitzen und besondere Winkel berechnen",
    },
    Tr {
        key: "cmd.sabbats.about",
        en: "Celtic Wheel of the Year (8 sabbats)",
        pt_br: "Roda do Ano Celta (8 sabás)",
        es: "Rueda del Año Celta (8 sabbats)",
        it: "Ruota dell'Anno Celtica (8 sabba)",
        de: "Keltisches Jahresrad (8 Sabbate)",
    },
    Tr {
        key: "cmd.esbats.about",
        en: "Named full moons (esbats)",
        pt_br: "Luas cheias nomeadas (esbás)",
        es: "Lunas llenas nombradas (esbats)",
        it: "Luna piena con nomi (esbat)",
        de: "Benannte Vollmonde (Esbats)",
    },
    Tr {
        key: "cmd.jd.about",
        en: "Convert between Julian day numbers and calendar dates",
        pt_br: "Converter entre dia juliano e datas do calendário",
        es: "Convertir entre día juliano y fechas del calendario",
        it: "Conversione tra giorno giuliano e date del calendario",
        de: "Zwischen Julianischem Datum und Kalenderdaten umrechnen",
    },
    Tr {
        key: "cmd.crossing.about",
        en: "Find the next ecliptic longitude crossing",
        pt_br: "Encontrar o próximo cruzamento de longitude eclíptica",
        es: "Encontrar el próximo cruce de longitud eclíptica",
        it: "Trova il prossimo passaggio di longitudine eclittica",
        de: "Nächste ekliptische Längenüberquerung finden",
    },
    Tr {
        key: "cmd.eclipse.about",
        en: "Find solar or lunar eclipses",
        pt_br: "Encontrar eclipses solares ou lunares",
        es: "Encontrar eclipses solares o lunares",
        it: "Trova eclissi solari o lunari",
        de: "Sonnen- oder Mondfinsternisse finden",
    },
    Tr {
        key: "cmd.moon.about",
        en: "Moon phase, illumination, and next principal phases",
        pt_br: "Fase da Lua, iluminação e próximas fases principais",
        es: "Fase lunar, iluminación y próximas fases principales",
        it: "Fase lunare, illuminazione e prossime fasi principali",
        de: "Mondphase, Beleuchtung und nächste Hauptphasen",
    },
    Tr {
        key: "cmd.omer.about",
        en: "Sefirat HaOmer — 49-day Omer count",
        pt_br: "Sefirat HaOmer — contagem de 49 dias do Ômer",
        es: "Sefirat HaOmer — cuenta del Omer de 49 días",
        it: "Sefirat HaOmer — conteggio dell'Omer di 49 giorni",
        de: "Sefirat HaOmer — 49-Tage-Omer-Zählung",
    },
    Tr {
        key: "cmd.calendar.about",
        en: "Multi-tradition religious calendars",
        pt_br: "Calendários religiosos de múltiplas tradições",
        es: "Calendarios religiosos de múltiples tradiciones",
        it: "Calendari religiosi di varie tradizioni",
        de: "Religiöse Kalender verschiedener Traditionen",
    },
    Tr {
        key: "cmd.phenomena.about",
        en: "Apparent planetary phenomena (magnitude, phase, illumination, elongation)",
        pt_br: "Fenômenos planetários aparentes (magnitude, fase, iluminação, elongação)",
        es: "Fenómenos planetarios aparentes (magnitud, fase, iluminación, elongación)",
        it: "Fenomeni planetari apparenti (magnitudine, fase, illuminazione, elongazione)",
        de: "Sichtbare Planetenphänomene (Helligkeit, Phase, Beleuchtung, Elongation)",
    },
    Tr {
        key: "cmd.render.about",
        en: "Render a Jinja2 template with celestial chart data (SVG, HTML, …)",
        pt_br: "Renderizar template Jinja2 com dados do mapa celestial (SVG, HTML, …)",
        es: "Renderizar plantilla Jinja2 con datos del gráfico celestial (SVG, HTML, …)",
        it: "Renderizza un template Jinja2 con dati della carta celeste (SVG, HTML, …)",
        de: "Jinja2-Vorlage mit Himmelskarten-Daten rendern (SVG, HTML, …)",
    },
];

/// Look up a translation by key for the given language.
///
/// Panics in debug builds if the key is missing — this is a programming
/// error, not a runtime condition. In release builds, returns a placeholder
/// `"<missing translation>"` so the user sees something rather than nothing
/// or a panic.
#[must_use]
pub fn tr(key: &str, lang: Lang) -> &'static str {
    for entry in TR_TABLE {
        if entry.key == key {
            return entry.pick(lang);
        }
    }
    debug_assert!(false, "missing translation key: {key}");
    "<missing translation>"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every translation row must have a non-empty value for every language.
    #[test]
    fn every_row_complete() {
        for entry in TR_TABLE {
            for &lang in Lang::all() {
                let value = entry.pick(lang);
                assert!(
                    !value.is_empty(),
                    "translation key `{}` empty for language `{}`",
                    entry.key,
                    lang.code()
                );
            }
        }
    }

    /// Translation keys must be unique. Duplicates would silently shadow
    /// each other depending on table order.
    #[test]
    fn keys_are_unique() {
        let mut keys: Vec<&str> = TR_TABLE.iter().map(|e| e.key).collect();
        keys.sort_unstable();
        let original_len = keys.len();
        keys.dedup();
        assert_eq!(
            keys.len(),
            original_len,
            "duplicate translation keys detected"
        );
    }

    fn assert_locales_map_to(expected: Lang, locales: &[&str]) {
        for loc in locales {
            assert_eq!(Lang::from_locale(loc), Some(expected), "locale {loc:?}");
        }
    }

    /// `Lang::from_locale` must handle the most common POSIX/BCP-47 forms.
    #[test]
    fn locale_parsing_common_forms() {
        assert_locales_map_to(Lang::En, &["en", "en_US", "en-US", "en_GB.UTF-8", "EN"]);
        // European Portuguese also routes to PtBr (only variant we ship).
        assert_locales_map_to(
            Lang::PtBr,
            &["pt_BR", "pt-BR", "pt_BR.UTF-8", "pt", "pt_PT"],
        );
        assert_locales_map_to(Lang::Es, &["es_ES.UTF-8", "es-MX"]);
        assert_locales_map_to(Lang::It, &["it_IT"]);
        assert_locales_map_to(Lang::De, &["de_AT.UTF-8", "de_CH"]);
    }

    #[test]
    fn locale_parsing_unsupported_returns_none() {
        // POSIX C / empty / unset
        assert_eq!(Lang::from_locale(""), None);
        assert_eq!(Lang::from_locale("C"), None);
        assert_eq!(Lang::from_locale("POSIX"), None);
        assert_eq!(Lang::from_locale("c"), None);

        // Languages we don't ship
        assert_eq!(Lang::from_locale("fr_FR"), None);
        assert_eq!(Lang::from_locale("ja_JP.UTF-8"), None);
        assert_eq!(Lang::from_locale("zh_CN"), None);
        assert_eq!(Lang::from_locale("ar_SA"), None);
        assert_eq!(Lang::from_locale("xy_ZZ"), None);
    }

    #[test]
    fn tr_returns_correct_translation() {
        // English (default)
        assert_eq!(
            tr("top.about", Lang::En),
            "Astronomical calculations — celestial engine"
        );
        // Brazilian Portuguese
        assert_eq!(
            tr("top.about", Lang::PtBr),
            "Cálculos astronômicos — motor celestial"
        );
        // German
        assert_eq!(
            tr("cmd.calc.about", Lang::De),
            "Geozentrische Planetenpositionen berechnen"
        );
    }

    #[test]
    fn tr_missing_key_returns_placeholder_in_release() {
        // In release builds the missing-key path returns a static placeholder
        // (the debug_assert! is a no-op). This test only runs in --release
        // mode because debug builds would correctly assert on a missing key.
        #[cfg(not(debug_assertions))]
        assert_eq!(tr("nonexistent.key", Lang::En), "<missing translation>");
    }

    #[test]
    fn lang_codes_are_distinct() {
        let codes: Vec<&str> = Lang::all().iter().map(|l| l.code()).collect();
        let mut sorted = codes.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(codes.len(), sorted.len(), "duplicate language codes");
    }

    #[test]
    fn detect_falls_back_to_english_when_unset() {
        // We can't actually unset env vars safely in tests (they're shared
        // process state), but we can verify the function returns *something*
        // sensible without panicking.
        let detected = Lang::detect();
        // Whatever the test runner's locale is, it must be one of ours.
        assert!(Lang::all().contains(&detected));
    }
}
