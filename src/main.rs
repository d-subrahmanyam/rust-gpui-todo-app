mod db;
mod model;
mod ui;

use gpui_kit::*;

use db::TodoRepo;
use ui::app::TodoApp;

fn main() {
    let repo = match TodoRepo::default_path()
        .map_err(|e| e.to_string())
        .and_then(|p| TodoRepo::open(&p).map_err(|e| format!("{}: {e}", p.display())))
    {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Failed to open the todo database: {e}");
            std::process::exit(1);
        }
    };

    application().with_assets(assets::Assets).run(move |cx| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(640.), px(720.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Todos".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        open_window(options, cx, |window, cx| cx.new(|cx| TodoApp::new(repo, window, cx)))
            .expect("Failed to open window");
    });
}
