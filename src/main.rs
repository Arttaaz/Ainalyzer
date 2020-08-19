extern crate pretty_env_logger;
#[macro_use] 
extern crate log;

use druid::widget::{Align, Flex, Label};
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Widget, WindowDesc};

mod goban;
use goban::Goban;

const HORIZONTAL_WIDGET_SPACING: f64 = 0.1;
const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const WINDOW_TITLE: LocalizedString<RootState> = LocalizedString::new("AInalyzer!");

#[derive(Debug, Clone, Data, Lens)]
struct RootState {
    text: String,
}

fn main() {
    pretty_env_logger::init();
    info!("Starting the app");
    let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .with_min_size((400.0, 400.0))
        .window_size((1280.0, 720.0));

    AppLauncher::with_window(main_window)
        .launch(RootState{
            text: "AInalyzer".to_string(),
        })
        .expect("failed to launch app");

    
}

fn build_root_widget() -> impl Widget<RootState> {
    let label = Label::new(|data: &RootState, _env: &Env| format!("{}", data.text));
    let layout = Flex::column()
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_flex_child(Goban {}, 1.0)
        .with_spacer(VERTICAL_WIDGET_SPACING);

    let layout_centered = Flex::row()
        .with_flex_spacer(HORIZONTAL_WIDGET_SPACING)
        .with_flex_child(layout, 1.0)
        .with_flex_spacer(HORIZONTAL_WIDGET_SPACING);

    Align::centered(layout_centered)
}

