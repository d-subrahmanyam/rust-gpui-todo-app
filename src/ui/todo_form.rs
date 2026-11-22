use chrono::NaiveDate;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    calendar::Date,
    date_picker::{DatePicker, DatePickerState},
    input::{Input, InputState},
    label::Label,
    v_flex, h_flex, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::{validate, NewTodo, Todo};

pub enum TodoFormEvent {
    Submit(NewTodo),
    Cancel,
}

pub struct TodoForm {
    title: Entity<InputState>,
    description: Entity<InputState>,
    date: Entity<DatePickerState>,
    editing: bool,
    error: Option<String>,
}

impl EventEmitter<TodoFormEvent> for TodoForm {}

impl TodoForm {
    pub fn new(existing: Option<&Todo>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (title, description, date) = match existing {
            Some(t) => (t.title.clone(), t.description.clone(), Some(t.target_date)),
            None => (String::new(), String::new(), None),
        };
        let title = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("What needs doing?")
                .default_value(title)
        });
        let description = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Description (optional)")
                .default_value(description)
        });
        let date = cx.new(|cx| {
            let mut s = DatePickerState::new(window, cx);
            if let Some(d) = date {
                s.set_date(d, window, cx);
            }
            s
        });
        Self { title, description, date, editing: existing.is_some(), error: None }
    }

    fn selected_date(&self, cx: &App) -> Option<NaiveDate> {
        match self.date.read(cx).date() {
            Date::Single(d) => d,
            Date::Range(d, _) => d,
        }
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        let title = self.title.read(cx).value();
        let description = self.description.read(cx).value();
        match validate(&title, &description, self.selected_date(cx)) {
            Ok(t) => {
                self.error = None;
                cx.emit(TodoFormEvent::Submit(t));
            }
            Err(e) => self.error = Some(e.message().to_string()),
        }
        cx.notify();
    }
}

impl Render for TodoForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let heading = if self.editing { "Edit todo" } else { "New todo" };
        v_flex()
            .gap_2()
            .p_3()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_md()
            .child(Label::new(heading).font_weight(FontWeight::SEMIBOLD))
            .child(Input::new(&self.title))
            .child(Input::new(&self.description))
            .child(DatePicker::new(&self.date).placeholder("Target completion date"))
            .when_some(self.error.clone(), |el, err| {
                el.child(div().text_color(cx.theme().danger).child(err))
            })
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("form-cancel")
                            .ghost()
                            .label("Cancel")
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(TodoFormEvent::Cancel))),
                    )
                    .child(
                        Button::new("form-save")
                            .primary()
                            .label("Save")
                            .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
                    ),
            )
    }
}
