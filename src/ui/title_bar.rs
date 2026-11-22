//! Custom title bar: app title, drag-to-move, minimize and close.
//! No maximize button, and double-click does not zoom the window.

use gpui_kit::component::{
    ActiveTheme, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::app::TodoApp;

pub const TITLE: &str = "Todos";

/// Window options for a client-decorated window that draws its own title bar.
pub fn window_options(bounds: Bounds<Pixels>) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some(TITLE.into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(9.0), px(9.0))),
        }),
        window_decorations: Some(WindowDecorations::Client),
        app_owns_titlebar_drag: true,
        window_min_size: Some(size(px(480.), px(520.))),
        ..Default::default()
    }
}

impl TodoApp {
    pub(super) fn render_title_bar(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        // macOS draws its own traffic lights; under server-side decorations the
        // window manager already provides controls. Only draw ours otherwise.
        let draw_controls = !cfg!(target_os = "macos")
            && matches!(window.window_decorations(), Decorations::Client { .. });
        let controls = window.window_controls();

        h_flex()
            .id("title-bar")
            .flex_shrink_0()
            .h(px(36.))
            .w_full()
            .justify_between()
            .items_center()
            .when(cfg!(target_os = "macos"), |el| el.pl(px(80.)))
            .when(!cfg!(target_os = "macos"), |el| el.pl_3())
            .border_b_1()
            .border_color(cx.theme().title_bar_border)
            .bg(cx.theme().title_bar)
            .child(
                h_flex()
                    .id("title-drag")
                    .flex_1()
                    .h_full()
                    .items_center()
                    .window_control_area(WindowControlArea::Drag)
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move())
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(format!("✅ {TITLE}")),
            )
            .when(draw_controls, |el| {
                el.child(
                    h_flex()
                        .gap_1()
                        .pr_1()
                        .when(controls.minimize, |el| {
                            el.child(
                                Button::new("win-minimize")
                                    .ghost()
                                    .small()
                                    .icon(IconName::WindowMinimize)
                                    .tooltip("Minimize")
                                    .on_click(|_, window, _| window.minimize_window()),
                            )
                        })
                        .child(
                            Button::new("win-close")
                                .ghost()
                                .small()
                                .icon(IconName::WindowClose)
                                .tooltip("Close")
                                .on_click(|_, _, cx| cx.quit()),
                        ),
                )
            })
    }
}
