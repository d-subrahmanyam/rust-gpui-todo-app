use chrono::NaiveDate;
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    calendar::Date,
    date_picker::{DatePicker, DatePickerState},
    h_flex,
    input::{Input, InputState, Textarea, TextareaState},
    label::Label,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::markdown::{MdFormat, apply};
use crate::model::{NewTodo, Todo, validate};

pub enum TodoFormEvent {
    Submit(NewTodo),
    Cancel,
}

pub struct TodoForm {
    title: Entity<InputState>,
    description: Entity<TextareaState>,
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
            TextareaState::new(window, cx)
                .auto_grow(4, 12)
                .placeholder("Description (optional, Markdown supported)")
                .default_value(description)
        });
        let date = cx.new(|cx| {
            let mut s = DatePickerState::new(window, cx);
            if let Some(d) = date {
                s.set_date(d, window, cx);
            }
            s
        });
        Self {
            title,
            description,
            date,
            editing: existing.is_some(),
            error: None,
        }
    }

    fn selected_date(&self, cx: &App) -> Option<NaiveDate> {
        match self.date.read(cx).date() {
            Date::Single(d) => d,
            Date::Range(d, _) => d,
        }
    }

    /// Apply a toolbar format to the description's current selection.
    /// Uses select + replace so the change is a single undoable edit.
    fn format(&mut self, f: MdFormat, window: &mut Window, cx: &mut Context<Self>) {
        self.description.update(cx, |state, cx| {
            let text = state.value();
            let edit = apply(&text, state.selected_range(), f);
            state.set_selected_range(edit.range.clone(), cx);
            state.replace(edit.text, window, cx);
            state.set_selected_range(edit.select, cx);
            state.focus(window, cx);
        });
    }

    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        const BUTTONS: [(&str, &str, &str, MdFormat); 8] = [
            ("md-bold", "B", "Bold", MdFormat::Bold),
            ("md-italic", "I", "Italic", MdFormat::Italic),
            ("md-code", "</>", "Inline code", MdFormat::Code),
            ("md-link", "🔗", "Link", MdFormat::Link),
            ("md-heading", "H", "Heading", MdFormat::Heading),
            ("md-bullet", "•", "Bulleted list", MdFormat::Bullet),
            ("md-numbered", "1.", "Numbered list", MdFormat::Numbered),
            ("md-quote", "❝", "Quote", MdFormat::Quote),
        ];
        let mut bar = h_flex()
            .gap_1()
            .px_1()
            .py_0p5()
            .rounded_t_md()
            .border_1()
            .border_b_0()
            .border_color(cx.theme().border)
            .bg(cx.theme().muted);
        for (id, label, tip, f) in BUTTONS {
            bar = bar.child(
                Button::new(id)
                    .ghost()
                    .compact()
                    .label(label)
                    .tooltip(tip)
                    .on_click(cx.listener(move |this, _, window, cx| this.format(f, window, cx))),
            );
        }
        bar
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
        let heading = if self.editing {
            "Edit todo"
        } else {
            "New todo"
        };
        v_flex()
            .gap_2()
            .p_3()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_md()
            .child(Label::new(heading).font_weight(FontWeight::SEMIBOLD))
            .child(Input::new(&self.title))
            .child(
                v_flex()
                    .child(self.render_toolbar(cx))
                    .child(Textarea::new(&self.description)),
            )
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
