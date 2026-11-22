use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::model::{NewTodo, Todo};

pub struct TodoRepo {
    conn: Connection,
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS todos (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  title        TEXT    NOT NULL,
  description  TEXT    NOT NULL DEFAULT '',
  target_date  TEXT    NOT NULL,
  completed    INTEGER NOT NULL DEFAULT 0,
  created_at   TEXT    NOT NULL,
  completed_at TEXT
);
CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
";

const DATE_FMT: &str = "%Y-%m-%d";

impl TodoRepo {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> rusqlite::Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// `<data dir>/gpui-hello/todos.db`, creating the parent directory.
    pub fn default_path() -> std::io::Result<PathBuf> {
        let dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gpui-hello");
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("todos.db"))
    }

    pub fn list(&self) -> rusqlite::Result<Vec<Todo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, target_date, completed, created_at, completed_at
             FROM todos ORDER BY created_at DESC, id DESC",
        )?;
        stmt.query_map([], row_to_todo)?.collect()
    }

    pub fn add(&self, t: &NewTodo) -> rusqlite::Result<i64> {
        self.conn.execute(
            "INSERT INTO todos (title, description, target_date, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![t.title, t.description, t.target_date.format(DATE_FMT).to_string(), now()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update(&self, id: i64, t: &NewTodo) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE todos SET title = ?1, description = ?2, target_date = ?3 WHERE id = ?4",
            params![t.title, t.description, t.target_date.format(DATE_FMT).to_string(), id],
        )?;
        Ok(())
    }

    pub fn complete(&self, id: i64) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE todos SET completed = 1, completed_at = ?1 WHERE id = ?2",
            params![now(), id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM todos WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> rusqlite::Result<Option<String>> {
        self.conn
            .query_row("SELECT value FROM settings WHERE key = ?1", params![key], |r| r.get(0))
            .optional()
    }

    pub fn set_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

fn now() -> String {
    // Microsecond precision keeps rapid inserts ordered by created_at.
    Local::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, false)
}

fn conv_err(idx: usize, e: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(idx, rusqlite::types::Type::Text, Box::new(e))
}

fn parse_ts(idx: usize, s: &str) -> rusqlite::Result<DateTime<Local>> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Local))
        .map_err(|e| conv_err(idx, e))
}

fn row_to_todo(r: &Row) -> rusqlite::Result<Todo> {
    let date: String = r.get(3)?;
    let created: String = r.get(5)?;
    let completed_at: Option<String> = r.get(6)?;
    Ok(Todo {
        id: r.get(0)?,
        title: r.get(1)?,
        description: r.get(2)?,
        target_date: NaiveDate::parse_from_str(&date, DATE_FMT).map_err(|e| conv_err(3, e))?,
        completed: r.get::<_, i64>(4)? != 0,
        created_at: parse_ts(5, &created)?,
        completed_at: completed_at.map(|s| parse_ts(6, &s)).transpose()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new(title: &str) -> NewTodo {
        NewTodo {
            title: title.into(),
            description: format!("{title} desc"),
            target_date: NaiveDate::from_ymd_opt(2026, 11, 30).unwrap(),
        }
    }

    #[test]
    fn add_then_list_newest_first() {
        let repo = TodoRepo::open_in_memory().unwrap();
        let a = repo.add(&new("A")).unwrap();
        let b = repo.add(&new("B")).unwrap();
        let list = repo.list().unwrap();
        assert_eq!(list.iter().map(|t| t.id).collect::<Vec<_>>(), vec![b, a]);
        assert_eq!(list[1].title, "A");
        assert_eq!(list[1].description, "A desc");
        assert_eq!(list[1].target_date, NaiveDate::from_ymd_opt(2026, 11, 30).unwrap());
        assert!(!list[1].completed);
        assert!(list[1].completed_at.is_none());
    }

    #[test]
    fn update_keeps_created_at_and_position() {
        let repo = TodoRepo::open_in_memory().unwrap();
        let a = repo.add(&new("A")).unwrap();
        let b = repo.add(&new("B")).unwrap();
        let before = repo.list().unwrap()[1].created_at;
        let changed = NewTodo {
            title: "A2".into(),
            description: "new".into(),
            target_date: NaiveDate::from_ymd_opt(2027, 1, 2).unwrap(),
        };
        repo.update(a, &changed).unwrap();
        let list = repo.list().unwrap();
        assert_eq!(list.iter().map(|t| t.id).collect::<Vec<_>>(), vec![b, a]);
        assert_eq!(list[1].title, "A2");
        assert_eq!(list[1].description, "new");
        assert_eq!(list[1].target_date, changed.target_date);
        assert_eq!(list[1].created_at, before);
    }

    #[test]
    fn complete_sets_flag_and_timestamp() {
        let repo = TodoRepo::open_in_memory().unwrap();
        let a = repo.add(&new("A")).unwrap();
        repo.complete(a).unwrap();
        let t = &repo.list().unwrap()[0];
        assert!(t.completed);
        assert!(t.completed_at.is_some());
    }

    #[test]
    fn delete_removes_row() {
        let repo = TodoRepo::open_in_memory().unwrap();
        let a = repo.add(&new("A")).unwrap();
        let b = repo.add(&new("B")).unwrap();
        repo.delete(a).unwrap();
        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, b);
    }

    #[test]
    fn unicode_and_quotes_round_trip() {
        let repo = TodoRepo::open_in_memory().unwrap();
        let t = NewTodo { title: "It's \"done\" ✅".into(), ..new("x") };
        repo.add(&t).unwrap();
        assert_eq!(repo.list().unwrap()[0].title, "It's \"done\" ✅");
    }

    #[test]
    fn reopen_file_keeps_rows() {
        let path = std::env::temp_dir().join(format!("todo-test-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        {
            let repo = TodoRepo::open(&path).unwrap();
            repo.add(&new("A")).unwrap();
        }
        let repo = TodoRepo::open(&path).unwrap();
        assert_eq!(repo.list().unwrap().len(), 1);
        drop(repo);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn settings_round_trip_and_upsert() {
        let repo = TodoRepo::open_in_memory().unwrap();
        assert_eq!(repo.get_setting("show_completed").unwrap(), None);
        repo.set_setting("show_completed", "0").unwrap();
        repo.set_setting("show_completed", "1").unwrap();
        assert_eq!(repo.get_setting("show_completed").unwrap(), Some("1".into()));
    }
}
