use std::sync::Arc;
use log::info;
use log::debug;
use druid::widget::{Flex, Label};
use druid::{AppLauncher, AppDelegate, Data, DelegateCtx, Event, Env, KbKey, KeyEvent, Lens, LocalizedString, Widget, WindowDesc, WindowId};

mod goban;
use goban::Goban;

mod history;
use history::History;

const HORIZONTAL_WIDGET_SPACING: f64 = 0.1; // Flex factor
const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const WINDOW_TITLE: LocalizedString<RootState> = LocalizedString::new("AInalyzer!");

#[derive(Debug, Clone, Copy, Data, PartialEq)]
pub enum Player {
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

#[derive(Clone, Data, Lens)]
struct RootState {
    text: String,
    pub turn: Player,
    pub history: Arc<Box<History>>,
}

struct Delegate;

impl AppDelegate<RootState> for Delegate {
    fn event(&mut self, ctx: &mut DelegateCtx, _window_id: WindowId, event: Event, _data: &mut RootState, _env: &Env) -> Option<Event> {
        match event.clone() {
            Event::KeyUp(KeyEvent {
                key: code,
                ..
            }) => match code {
                KbKey::Character(s) if *s == "q".to_string() => {
                    ctx.submit_command(druid::commands::QUIT_APP);
                    Some(event)
                },
                /*I want the code to open files here but apparently it doesn't work when the
                * command is sent here */
                //KbKey::Character(s) if *s == "o".to_string() => {
                //    ctx.submit_command(druid::Command::new(druid::commands::SHOW_OPEN_PANEL, druid::FileDialogOptions::new(), druid::Target::Auto));
                //    debug!("hello");
                //    Some(event)
                //},
                _ => Some(event),
            },
            _ => Some(event),
        }
    }

    fn command(&mut self, _ctx: &mut DelegateCtx, _target: druid::Target, cmd: &druid::Command, data: &mut RootState, _env: &Env) -> bool {
        if let Some(file) = cmd.get(druid::commands::OPEN_FILE) {
            let sgf = std::fs::read_to_string(file.path()).expect("failed to load sgf");
            let game = sgf_parser::parse(sgf.as_str()).expect("failed to parse sgf");
            //TODO: Build history tree from sgf
            //data.history = Arc::new(Box::new(History {tree: game, current_node: game.iter().next().unwrap().clone()}));
            debug!("{:?}", game);
        }
        false
    }
}

fn main() {
    scrub_log::init().unwrap();
    info!("Starting the app");
    let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .with_min_size((400.0, 400.0))
        .window_size((1280.0, 720.0));

    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .launch(RootState{
            text: "AInalyzer".to_string(),
            turn: Player::Black,
            history: Arc::new(Box::new(History::default())),
        })
        .expect("failed to launch app");
}

fn build_root_widget() -> impl Widget<RootState> {
    let label = Label::new(|data: &RootState, _env: &Env| format!("{}", data.text));
    let layout = Flex::column()
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_flex_child(Goban::default(), 1.0)
        .with_spacer(VERTICAL_WIDGET_SPACING);

    Flex::row()
        .with_flex_spacer(HORIZONTAL_WIDGET_SPACING)
        .with_flex_child(layout, 1.0)
        .with_flex_spacer(HORIZONTAL_WIDGET_SPACING)

}
