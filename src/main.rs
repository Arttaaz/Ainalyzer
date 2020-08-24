use log::info;
use druid::widget::{Flex, Label};
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Widget, WindowDesc};
use druid::Point;

mod goban;
use goban::Goban;
use goban::Stone;
const HORIZONTAL_WIDGET_SPACING: f64 = 0.1; // Flex factor
const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const WINDOW_TITLE: LocalizedString<RootState> = LocalizedString::new("AInalyzer!");

#[derive(Debug, Clone, Data, PartialEq)]
enum Player {
    Black,
    White,
}

impl Player {
    fn next(&mut self) {
        match self {
            Self::Black => *self = Self::White,
            Self::White => *self = Self::Black,
        }
    }
}

#[derive(Debug, Clone, Data, Lens)]
struct RootState {
    text: String,
    pub turn: Player,
}

fn main() {
    scrub_log::init().unwrap();
    info!("Starting the app");
    let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .with_min_size((400.0, 400.0))
        .window_size((1280.0, 720.0));

    AppLauncher::with_window(main_window)
        .launch(RootState{
            text: "AInalyzer".to_string(),
            turn: Player::Black,
        })
        .expect("failed to launch app");
}

fn build_root_widget() -> impl Widget<RootState> {
    let label = Label::new(|data: &RootState, _env: &Env| format!("{}", data.text));
    let layout = Flex::column()
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_flex_child(Goban {
                            stones: vec![Stone::black(Point::new(10.0, 10.0)), Stone::black(Point::new(9.0, 9.0)),
                                         Stone::white(Point::new(10.0, 9.0)), Stone::white(Point::new(9.0, 10.0))],
                            hover: None,
                        }, 1.0)
        .with_spacer(VERTICAL_WIDGET_SPACING);

    Flex::row()
        .with_flex_spacer(HORIZONTAL_WIDGET_SPACING)
        .with_flex_child(layout, 1.0)
        .with_flex_spacer(HORIZONTAL_WIDGET_SPACING)

}

