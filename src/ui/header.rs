use chrono::{Local, Timelike};
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    switch::Switch,
    v_flex,
};
use gpui_kit::*;

use super::app::TodoApp;
use super::format::{greeting, open_count_text};
use super::theme::toggle_label;

impl TodoApp {
    pub(super) fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.todos.iter().filter(|t| !t.completed).count();
        h_flex()
            .justify_between()
            .items_start()
            .gap_4()
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        Label::new(greeting(Local::now().hour()))
                            .text_xl()
                            .font_weight(FontWeight::BOLD),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child(open_count_text(open)),
                    ),
            )
            .child(
                v_flex()
                    .items_end()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("toggle-theme")
                                    .ghost()
                                    .label(toggle_label(cx.theme().is_dark()))
                                    .tooltip("Switch light / dark mode")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.toggle_theme(window, cx)
                                    })),
                            )
                            .child(
                                Button::new("new-todo")
                                    .primary()
                                    .label("+ New todo")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.open_form(None, window, cx)
                                    })),
                            ),
                    )
                    .child(
                        Switch::new("show-completed")
                            .label("Show completed")
                            .checked(self.show_completed)
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                this.set_show_completed(*checked, cx)
                            })),
                    ),
            )
    }
}
