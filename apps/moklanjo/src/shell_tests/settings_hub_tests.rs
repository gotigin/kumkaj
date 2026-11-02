#[cfg(test)]
mod settings_hub_tests {
    use super::*;

    /// Back on an open section returns to the hub; on the hub it leaves
    /// Settings; a section left open on Settings never eats Back elsewhere.
    #[test]
    fn back_closes_the_section_before_leaving_settings() {
        assert_eq!(settings_back_step(Screen::Settings, Some("printers")), Some(None));
        assert_eq!(settings_back_step(Screen::Settings, None), None);
        assert_eq!(settings_back_step(Screen::Pos, Some("printers")), None);
    }

    /// One row per section, each key once, and every row's words exist in
    /// every shipped language — a hub row must never read in English.
    #[test]
    fn every_hub_row_is_its_own_section_and_translated() {
        let keys: Vec<&str> = SETTINGS_SECTIONS.iter().map(|(k, ..)| *k).collect();
        let mut unique = keys.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), keys.len(), "{keys:?}");
        let labels = SETTINGS_SECTIONS
            .iter()
            .map(|(_, _, _, label)| *label)
            .chain(["sidebar_settings", "settings_hub_help"]);
        for label in labels {
            let english = locale::t("en", label);
            assert_ne!(english, label, "{label} has no English words");
            for lang in ["hi", "gu", "ar", "zh", "sw"] {
                assert_ne!(locale::t(lang, label), english, "{label} in {lang}");
            }
        }
    }
}
