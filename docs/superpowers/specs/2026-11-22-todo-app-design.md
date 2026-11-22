# Todo App — Design

Date: 2026-11-22
Project: `gpui-hello` (Rust, edition 2024, `gpui-kit = 0.10.0`)

## Intent

Turn the existing "Hello, World" GPUI window into a small, friendly desktop todo
app. Todos persist in a local SQLite database. The code should be modular so each
concern (data, storage, UI pieces) lives in its own module.

## Goals

- Add a todo with a **title**, **description**, and **target completion date**.
- List stored todos.
- Per todo: **edit**, **mark complete**, **delete**.
- Warm welcome message at the top; **today's date in a footer**.
- Completed todos remain visible, **faded and struck through**, with a
  **show/hide completed** toggle.
- Data survives restarts (SQLite).

## Non-goals

- Sync, accounts, multiple lists, tags, priorities, reminders/notifications.
- Search or filtering beyond the show/hide-completed toggle.
- Un-completing a todo is not required (see "Builder's calls").

## Anti-goals

- Losing data: a failed write must not silently drop a todo; show an error.
- Requiring the user to install SQLite separately.
- One giant `main.rs`.

## Layout (chosen: option B from the mockups)

```
┌──────────────────────────────────────────────────────────┐
│ Good morning! 👋 Let's get things done.   [+ New todo]   │
│ You have 3 open todos                [x] Show completed  │
├──────────────────────────────────────────────────────────┤
│ Buy groceries                                ✓  ✎  🗑    │
│ Milk, eggs, bread · due Nov 24                           │
│ Finish report                                ✓  ✎  🗑    │
│ Q4 summary · due Nov 20  (red: overdue)                  │
│ ~~Call plumber~~ (faded)                             🗑   │
├──────────────────────────────────────────────────────────┤
│ [ Form panel: Title / Description / Target date picker ] │
│ [                                   Cancel   Save      ] │
├──────────────────────────────────────────────────────────┤
│               Sunday, 22 November 2026                   │
└──────────────────────────────────────────────────────────┘
```

- **Header:** greeting varies by local time of day (morning / afternoon /
  evening) plus a count of open todos. "+ New todo" button on the right.
  "Show completed" toggle.
- **List:** newest first — the most recently added todo is at the top
  (by `created_at` descending, ties broken by `id` descending). Completed todos
  keep their position in that order (they are not moved). Overdue open todos show their date in red. Each row: title
  (bold), description + "due <date>" below. Actions: ✓ complete, ✎ edit,
  🗑 delete. Completed rows: faded, title struck through, only 🗑.
  Empty state: friendly "Nothing to do — enjoy your day!" message.
- **Form panel:** shown when "+ New todo" or ✎ is clicked; hidden otherwise.
  Fields: Title (text input, required), Description (text input, optional),
  Target date (gpui-component's built-in **DatePicker**, required). Buttons:
  Cancel, Save. In edit mode it is pre-filled and Save updates the row.
  Validation errors appear inline under the form.
- **Delete:** asks for confirmation (dialog or inline "Delete? Yes / No").
- **Footer:** today's date, e.g. "Sunday, 22 November 2026".

## How it's built (decided)

- UI: `gpui-kit` with its `gpui-component` layer (already in use).
- Database: `rusqlite` with the `bundled` feature (no system SQLite needed).
- Dates: `chrono` (`NaiveDate` for target date, `Local` for today/greeting).
- DB location: `dirs::data_dir()/gpui-hello/todos.db`
  (`~/.local/share/gpui-hello/todos.db` on Linux); directory created if missing.

### Schema

```sql
CREATE TABLE IF NOT EXISTS todos (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  title        TEXT    NOT NULL,
  description  TEXT    NOT NULL DEFAULT '',
  target_date  TEXT    NOT NULL,          -- ISO 8601 YYYY-MM-DD
  completed    INTEGER NOT NULL DEFAULT 0,
  created_at   TEXT    NOT NULL,          -- RFC 3339
  completed_at TEXT                        -- RFC 3339, NULL if open
);

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);  -- holds show_completed = '1' | '0'
```

### Module layout

```
src/
  main.rs            app bootstrap: open DB, init gpui-kit, open window
  model.rs           Todo, NewTodo, validation (title non-empty)
  db.rs              TodoRepo: open(path), open_in_memory(), list(),
                     add(), update(), complete(), delete(),
                     get_setting()/set_setting()
  ui/
    mod.rs
    app.rs           TodoApp root view: owns repo, todos, form state,
                     show_completed; reloads list after each mutation
    header.rs        greeting(now) + open count + buttons/toggle
    todo_list.rs     row rendering, sorting, overdue styling
    todo_form.rs     form entity (inputs + date picker), add/edit modes
    footer.rs        formatted today's date
```

Pure logic (validation, greeting text, sort order, overdue check, date
formatting) lives in plain functions so it can be unit-tested without a window.

### Testing

- `db.rs`: unit tests against an in-memory database covering add, list order
  (newest first),
  update, complete, delete, and settings round-trip.
- Pure UI helpers: greeting by hour, sort order, overdue check, footer format.
- UI is verified by running the app manually.

## Builder's calls (decided by Claude, not the user)

- Time-of-day greeting wording; open-todo count in header.
- Title and target date required; description optional.
- Delete confirmation.
- Overdue dates shown in red.
- Show-completed defaults to on and is remembered in the `settings` table.
- No "undo complete" — completed rows only offer delete. (Easy to add later.)
- DB path under the user data dir; `rusqlite` bundled; `chrono`, `dirs` crates.
- DB errors surface as an error message in the window, not a panic (except
  failure to open the DB at startup, which exits with a clear message).

## Left to the builder

- Exact gpui-component APIs (Input, DatePicker, Checkbox/Switch, Dialog) for
  `gpui-component 0.10.0`; adapt the layout if a component differs.
- Spacing, colours, and fonts, using the component theme defaults.
