# GPUI Todo

A small, friendly desktop todo app written in Rust with [GPUI](https://www.gpui.rs/)
(via [`gpui-kit`](https://crates.io/crates/gpui-kit)) and a local SQLite database.

## Features

- **Add todos** with a title, an optional multi-line **Markdown** description
  and a target completion date picked from a built-in calendar.
- **Markdown toolbar** above the description: bold, italic, inline code, link,
  heading, bulleted list, numbered list and quote. Buttons wrap the selected
  text (or insert a placeholder); list/heading/quote buttons apply to every
  selected line and toggle off when pressed again. Descriptions are rendered
  as Markdown in the list.
- **Newest first:** the most recently added todo sits at the top of the list.
- **Per-todo actions:**
  - ✓ mark complete
  - ✎ edit (opens the form pre-filled)
  - 🗑 delete, with an inline "Delete? Yes / No" confirmation
- **Completed todos** stay in the list, faded and struck through. A
  **Show completed** switch hides or shows them, and the choice is remembered.
- **Overdue dates** of open todos are shown in red.
- **Warm welcome:** a greeting that changes with the time of day, plus a count
  of open todos.
- **Light / dark mode:** a ☀️ / 🌙 button in the header switches themes. The
  choice is remembered; on first launch the app follows your system setting.
- **Custom title bar** with the app name, drag-to-move, **minimize** and
  **close** buttons. There is deliberately no maximize button. (On macOS the
  system traffic lights are used; under a window manager that draws its own
  decorations, its controls are used instead.)
- **Footer** shows today's date, e.g. "Sunday, 22 November 2026".
- **Validation:** a title and a target date are required; problems are shown
  inline in the form. Database errors are shown in the window instead of
  crashing the app.

## Requirements

- Rust (edition 2024, a recent stable toolchain)
- Linux, macOS or Windows with a desktop session. On Linux, the GPUI system
  libraries for X11/Wayland (e.g. `libxkbcommon`, `libvulkan`, `fontconfig`)
  must be installed.
- No system SQLite is needed: it is bundled through `rusqlite`.

> `.cargo/config.toml` adds `/home/linuxbrew/.linuxbrew/lib` to the linker
> search path for `x86_64-unknown-linux-gnu`. Adjust or remove it if your
> libraries live elsewhere.

## Running

```bash
cargo run            # start the app
cargo test           # run the unit tests
cargo build --release
```

## Where data is stored

Todos and settings live in a SQLite file in your user data directory:

| OS      | Path                                                   |
| ------- | ------------------------------------------------------ |
| Linux   | `~/.local/share/gpui-hello/todos.db`                   |
| macOS   | `~/Library/Application Support/gpui-hello/todos.db`    |
| Windows | `%APPDATA%\gpui-hello\todos.db`                        |

Delete that file to start fresh.

### Schema

```sql
todos(id, title, description, target_date /* YYYY-MM-DD */,
      completed, created_at /* RFC 3339 */, completed_at)
settings(key, value)   -- show_completed = '1'|'0', theme = 'light'|'dark'
```

## Project layout

New to the code? Start with **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)**. It
explains how the code is laid out and how each feature (title bar, Markdown
toolbar, theming, storage and so on) works.

```
src/
  main.rs          opens the database and the window
  model.rs         Todo / NewTodo types and input validation
  db.rs            TodoRepo: schema, CRUD and settings on SQLite
  ui/
    app.rs         TodoApp root view: state, actions, layout
    title_bar.rs   custom title bar (drag, minimize, close) and window options
    header.rs      greeting, open count, theme toggle, "+ New todo", switch
    todo_form.rs   add / edit form (title, Markdown description + toolbar, date picker)
    todo_list.rs   todo rows and their actions
    footer.rs      today's date
    format.rs      pure text/date helpers (unit-tested)
    theme.rs       light/dark setting helpers (unit-tested)
    markdown.rs    Markdown toolbar text edits (unit-tested)
docs/superpowers/  design spec and implementation plan
```

## Tests

`cargo test` covers validation, the SQLite repository (ordering, edits,
completion, deletion, unicode, reopening the file, settings) and the
formatting, theme and Markdown toolbar helpers. The UI itself is checked by running the app.
