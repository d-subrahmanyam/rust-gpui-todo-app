use chrono::{DateTime, Local, NaiveDate};

#[derive(Debug, Clone, PartialEq)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub target_date: NaiveDate,
    pub completed: bool,
    pub created_at: DateTime<Local>,
    pub completed_at: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewTodo {
    pub title: String,
    pub description: String,
    pub target_date: NaiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationError {
    TitleRequired,
    DateRequired,
}

impl ValidationError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::TitleRequired => "Title is required",
            Self::DateRequired => "Target date is required",
        }
    }
}

pub fn validate(
    title: &str,
    description: &str,
    date: Option<NaiveDate>,
) -> Result<NewTodo, ValidationError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(ValidationError::TitleRequired);
    }
    let target_date = date.ok_or(ValidationError::DateRequired)?;
    Ok(NewTodo {
        title: title.to_string(),
        description: description.trim().to_string(),
        target_date,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 30).unwrap()
    }

    #[test]
    fn validate_trims_and_accepts() {
        let t = validate("  Buy milk ", "  2 litres ", Some(d())).unwrap();
        assert_eq!(
            t,
            NewTodo { title: "Buy milk".into(), description: "2 litres".into(), target_date: d() }
        );
    }

    #[test]
    fn validate_accepts_empty_description() {
        let t = validate("Buy milk", "", Some(d())).unwrap();
        assert_eq!(t.description, "");
    }

    #[test]
    fn validate_rejects_blank_title() {
        assert_eq!(validate("   ", "x", Some(d())), Err(ValidationError::TitleRequired));
        assert_eq!(ValidationError::TitleRequired.message(), "Title is required");
    }

    #[test]
    fn validate_rejects_missing_date() {
        assert_eq!(validate("Buy milk", "", None), Err(ValidationError::DateRequired));
        assert_eq!(ValidationError::DateRequired.message(), "Target date is required");
    }
}
