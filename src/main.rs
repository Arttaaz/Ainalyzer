#[macro_use] extern crate lazy_static;

use std::sync::{Arc, Mutex};
use std::io::Write;
use log::info;
use druid::widget::Flex;
use druid::{AppLauncher, AppDelegate, Data, DelegateCtx, Event, Env, Handled, KbKey, KeyEvent, Lens, LocalizedString, Widget, WindowDesc, WindowId};

mod dialogs;

mod engine;
use engine::Engine;

mod goban;
use goban::Goban;

mod history;
use history::History;

mod selectors;
mod engine_commands;

const HORIZONTAL_WIDGET_SPACING: f64 = 0.01; // Flex factor
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

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Black => "B",
            Self::White => "W",
        })
    }
}

impl Into<sgf_parser::Color> for Player {
    fn into(self) -> sgf_parser::Color {
        match self {
            Self::Black => sgf_parser::Color::Black,
            Self::White => sgf_parser::Color::White,
        }
    }
}

// State machine
rust_fsm::state_machine! {
    derive(Debug, Clone)
    pub EngineState(Idle)

    Idle(StartAnalyze) => Analyzing,
    Analyzing(StopAnalyze) => Idle,
}

#[derive(Clone, Data, Lens)]
pub struct RootState {
    text: String,
    pub turn: Player,
    pub history: Arc<Box<History>>,
    pub path: Option<String>,
    pub engine: Arc<Mutex<libgtp::Controller>>,
    pub engine_state: Arc<Mutex<rust_fsm::StateMachine<EngineState>>>,
    pub analyze_info: Arc<Mutex<Option<libgtp::Info>>>,
    pub analyze_timer_token: Arc<Option<druid::TimerToken>>,
}

impl RootState {
    pub fn is_file_updated(&self) -> bool {
        if let Some(path) = &self.path {
            let file = std::fs::read_to_string(std::path::PathBuf::from(path)).expect("couldn't open file");
            let sgf: String = self.history.into_game_tree().into();
            
            file == sgf
        } else {
            false
        }
    }
}

struct Delegate;

impl AppDelegate<RootState> for Delegate {
    fn event(&mut self, ctx: &mut DelegateCtx, _window_id: WindowId, event: Event, data: &mut RootState, _env: &Env) -> Option<Event> {
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
            Event::Timer(t) => {
                if data.analyze_timer_token.is_some() {
                    if data.analyze_timer_token.unwrap() == t {
                        ctx.submit_command(selectors::ANALYZE_TIMER_TOKEN);
                        ctx.submit_command(selectors::DRAW_ANALYZE);
                    }
                }
                Some(event)
            },
            _ => Some(event),
        }
    }

    fn command(&mut self, _ctx: &mut DelegateCtx, _target: druid::Target, cmd: &druid::Command, data: &mut RootState, _env: &Env) -> Handled {
        if let Some(file) = cmd.get(druid::commands::OPEN_FILE) {
            data.path = Some(file.path().to_str().unwrap().to_string());
            let sgf = std::fs::read_to_string(file.path()).expect("failed to load sgf");
            let game = sgf_parser::parse(sgf.as_str()).expect("failed to parse sgf");
            data.history = Arc::new(Box::new(History::from(game)));
            Handled::No
        } else if let Some(file) = cmd.get(druid::commands::SAVE_FILE_AS) {
            data.path = Some(file.path().to_str().unwrap().to_owned());

            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(data.path.clone().unwrap())
                .expect("couldn't create/open file");

            let mut bufw = std::io::BufWriter::new(file);
            let sgf: String = data.history.into_game_tree().into();
            bufw.write_all(sgf.as_bytes()).expect("couldn't write to file");
            Handled::Yes
        } else if let Some(_) = cmd.get(druid::commands::CLOSE_WINDOW) {
            log::debug!("hey");
            Handled::No
        } else {
            Handled::No
        }
    }
}

fn main() {
    scrub_log::init().unwrap();
    info!("Starting the app");
    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .with_min_size((400.0, 400.0))
        .window_size((1280.0, 720.0));

    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .launch(RootState{
            text: "AInalyzer".to_string(),
            turn: Player::Black,
            history: Arc::new(Box::new(History::default())),
            path: None,
            engine: Arc::new(Mutex::new(Engine::engine_startup())),
            engine_state: Arc::new(Mutex::new(rust_fsm::StateMachine::new())),
            analyze_info: Arc::new(Mutex::new(None)),
            analyze_timer_token: Arc::new(None),
        })
        .expect("failed to launch app");
}

fn build_root_widget() -> impl Widget<RootState> {
    let layout = Flex::column()
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_flex_child(Goban::default(), 1.0)
        .with_spacer(VERTICAL_WIDGET_SPACING);

    Flex::row()
        .with_flex_spacer(HORIZONTAL_WIDGET_SPACING)
        .with_flex_child(Engine::build_engine_tab(), 0.2)
        .with_flex_spacer(HORIZONTAL_WIDGET_SPACING)
        .with_flex_child(layout, 1.0)
        .with_flex_spacer(HORIZONTAL_WIDGET_SPACING)

}
