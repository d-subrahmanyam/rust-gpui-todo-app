use chrono::NaiveDate;

use crate::model::Todo;

pub const EMPTY_TEXT: &str = "Nothing to do — enjoy your day!";

pub fn greeting(hour: u32) -> &'static str {
    match hour {
        5..=11 => "Good morning! 👋 Let's get things done.",
        12..=16 => "Good afternoon! ☀️ Let's keep it going.",
        _ => "Good evening! 🌙 Let's wrap things up.",
    }
}

pub fn open_count_text(n: usize) -> String {
    match n {
        0 => "No open todos".to_string(),
        1 => "You have 1 open todo".to_string(),
        n => format!("You have {n} open todos"),
    }
}

pub fn is_overdue(t: &Todo, today: NaiveDate) -> bool {
    !t.completed && t.target_date < today
}

pub fn footer_date(today: NaiveDate) -> String {
    today.format("%A, %-d %B %Y").to_string()
}

pub fn due_text(d: NaiveDate) -> String {
    d.format("due %b %-d").to_string()
}

pub fn visible(todos: &[Todo], show_completed: bool) -> Vec<&Todo> {
    todos
        .iter()
        .filter(|t| show_completed || !t.completed)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn todo(target: NaiveDate, completed: bool) -> Todo {
        Todo {
            id: 1,
            title: "t".into(),
            description: String::new(),
            target_date: target,
            completed,
            created_at: Local::now(),
            completed_at: None,
        }
    }

    #[test]
    fn greeting_by_hour() {
        assert_eq!(greeting(4), "Good evening! 🌙 Let's wrap things up.");
        assert_eq!(greeting(5), "Good morning! 👋 Let's get things done.");
        assert_eq!(greeting(11), "Good morning! 👋 Let's get things done.");
        assert_eq!(greeting(12), "Good afternoon! ☀️ Let's keep it going.");
        assert_eq!(greeting(16), "Good afternoon! ☀️ Let's keep it going.");
        assert_eq!(greeting(17), "Good evening! 🌙 Let's wrap things up.");
    }

    #[test]
    fn open_count_text_pluralises() {
        assert_eq!(open_count_text(0), "No open todos");
        assert_eq!(open_count_text(1), "You have 1 open todo");
        assert_eq!(open_count_text(3), "You have 3 open todos");
    }

    #[test]
    fn overdue_only_when_open_and_past() {
        let today = date(2026, 11, 22);
        assert!(is_overdue(&todo(date(2026, 11, 21), false), today));
        assert!(!is_overdue(&todo(today, false), today));
        assert!(!is_overdue(&todo(date(2026, 11, 21), true), today));
    }

    #[test]
    fn footer_and_due_formats() {
        assert_eq!(footer_date(date(2026, 11, 22)), "Sunday, 22 November 2026");
        assert_eq!(due_text(date(2026, 11, 24)), "due Nov 24");
    }

    #[test]
    fn visible_hides_completed_when_off() {
        let all_done = vec![todo(date(2026, 1, 1), true), todo(date(2026, 1, 2), true)];
        assert!(visible(&all_done, false).is_empty());
        assert_eq!(visible(&all_done, true).len(), 2);
        let mixed = vec![todo(date(2026, 1, 1), true), todo(date(2026, 1, 2), false)];
        let v = visible(&mixed, false);
        assert_eq!(v.len(), 1);
        assert!(!v[0].completed);
    }
}
