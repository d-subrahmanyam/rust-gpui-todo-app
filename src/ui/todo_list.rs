use chrono::{Local, NaiveDate};
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    h_flex, v_flex, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::app::TodoApp;
use super::format::{due_text, is_overdue, visible, EMPTY_TEXT};
use crate::model::Todo;

impl TodoApp {
    pub(super) fn render_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let today = Local::now().date_naive();
        let rows = visible(&self.todos, self.show_completed);
        let mut list = v_flex()
            .id("todo-list")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll();
        if rows.is_empty() {
            list = list.child(
                div()
                    .py_8()
                    .flex()
                    .justify_center()
                    .text_color(cx.theme().muted_foreground)
                    .child(EMPTY_TEXT),
            );
        }
        for t in rows {
            list = list.child(self.render_row(t, today, cx));
        }
        list
    }

    fn render_row(&self, t: &Todo, today: NaiveDate, cx: &mut Context<Self>) -> impl IntoElement {
        let id = t.id;
        let overdue = is_overdue(t, today);
        let meta = if t.description.is_empty() {
            due_text(t.target_date)
        } else {
            format!("{} · {}", t.description, due_text(t.target_date))
        };
        let title = div().font_weight(FontWeight::SEMIBOLD).child(t.title.clone());
        let title = if t.completed { title.line_through() } else { title };

        let actions = if self.confirm_delete == Some(id) {
            h_flex()
                .gap_1()
                .child(div().text_sm().child("Delete?"))
                .child(
                    Button::new(("del-yes", id as u64))
                        .danger()
                        .label("Yes")
                        .on_click(cx.listener(move |this, _, _, cx| this.delete(id, cx))),
                )
                .child(
                    Button::new(("del-no", id as u64))
                        .ghost()
                        .label("No")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.confirm_delete = None;
                            cx.notify();
                        })),
                )
        } else {
            let mut a = h_flex().gap_1();
            if !t.completed {
                a = a
                    .child(
                        Button::new(("done", id as u64))
                            .ghost()
                            .label("✓")
                            .tooltip("Mark complete")
                            .on_click(cx.listener(move |this, _, _, cx| this.complete(id, cx))),
                    )
                    .child(
                        Button::new(("edit", id as u64))
                            .ghost()
                            .label("✎")
                            .tooltip("Edit")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.open_form(Some(id), window, cx)
                            })),
                    );
            }
            a.child(
                Button::new(("delete", id as u64))
                    .ghost()
                    .label("🗑")
                    .tooltip("Delete")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.confirm_delete = Some(id);
                        cx.notify();
                    })),
            )
        };

        h_flex()
            .justify_between()
            .items_center()
            .gap_2()
            .py_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .when(t.completed, |el| el.opacity(0.5))
            .child(
                v_flex()
                    .gap_0p5()
                    .min_w_0()
                    .child(title)
                    .child(
                        div()
                            .text_sm()
                            .text_color(if overdue {
                                cx.theme().danger
                            } else {
                                cx.theme().muted_foreground
                            })
                            .child(meta),
                    ),
            )
            .child(actions)
    }
}
