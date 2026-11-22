use gpui_kit::component::{v_flex, ActiveTheme, Theme, ThemeMode};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::theme::{setting_value, stored_dark, THEME_KEY};
use super::todo_form::{TodoForm, TodoFormEvent};
use crate::db::TodoRepo;
use crate::model::{NewTodo, Todo};

const SHOW_COMPLETED: &str = "show_completed";

pub struct TodoApp {
    repo: TodoRepo,
    pub(super) todos: Vec<Todo>,
    pub(super) show_completed: bool,
    form: Option<Entity<TodoForm>>,
    editing: Option<i64>,
    pub(super) confirm_delete: Option<i64>,
    error: Option<String>,
    _form_sub: Option<Subscription>,
}

impl TodoApp {
    pub fn new(repo: TodoRepo, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut error = None;
        match repo.get_setting(THEME_KEY).map(|v| stored_dark(v.as_deref())) {
            Ok(Some(dark)) => Theme::change(mode(dark), Some(window), cx),
            Ok(None) => Theme::sync_system_appearance(Some(window), cx),
            Err(e) => error = Some(format!("Could not load settings: {e}")),
        }
        let show_completed = match repo.get_setting(SHOW_COMPLETED) {
            Ok(v) => v.as_deref() != Some("0"),
            Err(e) => {
                error = Some(format!("Could not load settings: {e}"));
                true
            }
        };
        let mut app = Self {
            repo,
            todos: Vec::new(),
            show_completed,
            form: None,
            editing: None,
            confirm_delete: None,
            error,
            _form_sub: None,
        };
        app.reload(cx);
        app
    }

    fn report<T>(&mut self, what: &str, r: rusqlite::Result<T>) -> Option<T> {
        match r {
            Ok(v) => Some(v),
            Err(e) => {
                self.error = Some(format!("Could not {what}: {e}"));
                None
            }
        }
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        if let Some(todos) = self.report("load todos", self.repo.list()) {
            self.todos = todos;
        }
        cx.notify();
    }

    pub(super) fn open_form(&mut self, id: Option<i64>, window: &mut Window, cx: &mut Context<Self>) {
        let existing = id.and_then(|id| self.todos.iter().find(|t| t.id == id).cloned());
        let form = cx.new(|cx| TodoForm::new(existing.as_ref(), window, cx));
        self._form_sub = Some(cx.subscribe(&form, |this, _, ev: &TodoFormEvent, cx| match ev {
            TodoFormEvent::Submit(t) => this.submit(t.clone(), cx),
            TodoFormEvent::Cancel => this.close_form(cx),
        }));
        self.form = Some(form);
        self.editing = id;
        self.confirm_delete = None;
        cx.notify();
    }

    fn close_form(&mut self, cx: &mut Context<Self>) {
        self.form = None;
        self.editing = None;
        self._form_sub = None;
        cx.notify();
    }

    fn submit(&mut self, t: NewTodo, cx: &mut Context<Self>) {
        let ok = match self.editing {
            Some(id) => self.report("update the todo", self.repo.update(id, &t)).is_some(),
            None => self.report("save the todo", self.repo.add(&t)).is_some(),
        };
        if ok {
            self.error = None;
            self.close_form(cx);
        }
        self.reload(cx);
    }

    pub(super) fn complete(&mut self, id: i64, cx: &mut Context<Self>) {
        let r = self.repo.complete(id);
        self.report("complete the todo", r);
        self.reload(cx);
    }

    pub(super) fn delete(&mut self, id: i64, cx: &mut Context<Self>) {
        let r = self.repo.delete(id);
        self.report("delete the todo", r);
        self.confirm_delete = None;
        if self.editing == Some(id) {
            self.close_form(cx);
        }
        self.reload(cx);
    }

    pub(super) fn toggle_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let dark = !cx.theme().is_dark();
        Theme::change(mode(dark), Some(window), cx);
        let r = self.repo.set_setting(THEME_KEY, setting_value(dark));
        self.report("save the theme", r);
        cx.notify();
    }

    pub(super) fn set_show_completed(&mut self, on: bool, cx: &mut Context<Self>) {
        self.show_completed = on;
        let r = self.repo.set_setting(SHOW_COMPLETED, if on { "1" } else { "0" });
        self.report("save the setting", r);
        cx.notify();
    }
}

fn mode(dark: bool) -> ThemeMode {
    if dark { ThemeMode::Dark } else { ThemeMode::Light }
}

impl Render for TodoApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .p_4()
            .gap_3()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_header(cx))
            .when_some(self.error.clone(), |el, err| {
                el.child(
                    div()
                        .p_2()
                        .rounded_md()
                        .bg(cx.theme().danger)
                        .text_color(cx.theme().danger_foreground)
                        .child(err),
                )
            })
            .when_some(self.form.clone(), |el, form| el.child(form))
            .child(self.render_list(cx))
            .child(self.render_footer(cx))
    }
}
