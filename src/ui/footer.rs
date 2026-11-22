use chrono::Local;
use gpui_kit::component::ActiveTheme;
use gpui_kit::*;

use super::app::TodoApp;
use super::format::footer_date;

impl TodoApp {
    pub(super) fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .pt_2()
            .border_t_1()
            .border_color(cx.theme().border)
            .flex()
            .justify_center()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .child(footer_date(Local::now().date_naive()))
    }
}
