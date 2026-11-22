//! Light/dark mode: persisted as the `theme` setting ("light" | "dark").

pub const THEME_KEY: &str = "theme";

/// `Some(true)` = dark, `Some(false)` = light, `None` = no valid choice saved
/// (follow the system appearance).
pub fn stored_dark(value: Option<&str>) -> Option<bool> {
    match value {
        Some("dark") => Some(true),
        Some("light") => Some(false),
        _ => None,
    }
}

pub fn setting_value(dark: bool) -> &'static str {
    if dark { "dark" } else { "light" }
}

/// Label for the header button: names the mode you switch *to*.
pub fn toggle_label(dark: bool) -> &'static str {
    if dark { "☀️ Light" } else { "🌙 Dark" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stored_value() {
        assert_eq!(stored_dark(Some("dark")), Some(true));
        assert_eq!(stored_dark(Some("light")), Some(false));
    }

    #[test]
    fn unknown_or_missing_value_means_follow_system() {
        assert_eq!(stored_dark(None), None);
        assert_eq!(stored_dark(Some("purple")), None);
    }

    #[test]
    fn setting_value_round_trips() {
        assert_eq!(stored_dark(Some(setting_value(true))), Some(true));
        assert_eq!(stored_dark(Some(setting_value(false))), Some(false));
    }

    #[test]
    fn toggle_label_offers_the_other_mode() {
        assert_eq!(toggle_label(true), "☀️ Light");
        assert_eq!(toggle_label(false), "🌙 Dark");
    }
}
