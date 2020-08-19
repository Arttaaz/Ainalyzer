use druid::widget::{Align, Flex, Label, Spinner};
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Widget, WindowDesc};

const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const WINDOW_TITLE: LocalizedString<HelloWidget> = LocalizedString::new("AInalyzer!");

#[derive(Debug, Clone, Data, Lens)]
struct HelloWidget {
    text: String,
}

fn main() {
   let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .window_size((400.0, 400.0));

    AppLauncher::with_window(main_window)
        .launch(HelloWidget{
            text: "loading the game analyzer app. Trust me it is loading.".to_string(),
        })
        .expect("failed to launch app");
}

fn build_root_widget() -> impl Widget<HelloWidget> {
    let label = Label::new(|data: &HelloWidget, _env: &Env| format!("{}", data.text));

    let loading = Spinner::new();

    let layout = Flex::column()
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(loading);

    Align::centered(layout)
}
