#[macro_use] extern crate lazy_static;

use std::io::Write;
use log::info;
use iced::{executor, Application, Command, Element, Settings, window};
use iced_native::{Widget, Event, keyboard::KeyCode};
use iced::widget::{column, row};
use native_dialog::{FileDialog, MessageDialog};

mod dialogs;
mod engine;
use engine::Engine;

mod goban;
use goban::Goban;

mod history;
mod selectors;
mod engine_commands;

#[derive(Debug, Clone, Copy, PartialEq)]
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

impl Into<iced::Color> for Player {
    fn into(self) -> iced::Color {
        match self {
            Self::Black => iced::Color::BLACK,
            Self::White => iced::Color::WHITE,
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

#[derive(Debug, Clone)]
pub enum GobanEvent {
    Play(goban::Point, goban::Stone),
    PreviousState,
    NextState,
}

#[derive(Debug, Clone)]
pub enum EngineCommand {
    EnginePlay(Player, goban::Point),
    EngineUndo,
}

#[derive(Debug, Clone)]
pub enum Message {
    Event(iced_native::Event),
    Goban(GobanEvent),
    EngineError,
    EngineTick(std::time::Instant),
    EngineCommand(EngineCommand),
    StartAnalyze,
    RefreshAnalyze,
    StopAnalyze,
    OpenFile(std::path::PathBuf),
    DialogCancel,
}

struct Ainalyzer {
    engine: Engine,
    engine_state: rust_fsm::StateMachine<EngineState>,
    goban: Goban,
    opened_file: Option<std::path::PathBuf>,
}

impl Application for Ainalyzer {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = iced::theme::Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self {
            engine: Engine::new(),
            engine_state: rust_fsm::StateMachine::new(),
            goban: Goban::default(),
            opened_file: None,
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("AInalyzer")
    }

    fn theme(&self) -> Self::Theme {
        Self::Theme::Dark
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Event(event) => {
                match event {
                    Event::Keyboard(iced_native::keyboard::Event::KeyReleased{ key_code, ..}) => {
                        match key_code {
                            KeyCode::Q => return window::close(),
                            KeyCode::O => {
                                return Command::perform(async move {
                                    FileDialog::new().show_open_single_file().expect("Open dialog failed")
                                }, |message| {
                                    match message {
                                        Some(m) => Message::OpenFile(m),
                                        None => Message::DialogCancel,
                                    }
                                })
                            },
                            KeyCode::N => {
                                self.goban = Goban::default();
                            }
                            KeyCode::S => {
                                match &self.opened_file {
                                    Some(path) => {
                                        let file = std::fs::OpenOptions::new()
                                            .write(true)
                                            .create(true)
                                            .open(path.clone())
                                            .expect("couldn't create/open file");

                                        let mut bufw = std::io::BufWriter::new(file);
                                        let sgf: String = self.goban.history.into_game_tree().into();
                                        bufw.write_all(sgf.as_bytes()).expect("couldn't write to file");
                                    },
                                    None => {
                                        let path = FileDialog::new().show_save_single_file().expect("Save dialog failed");
                                        match path {
                                            Some(p) => {
                                                self.opened_file = Some(p.clone());
                                                let file = std::fs::OpenOptions::new()
                                                    .write(true)
                                                    .create(true)
                                                    .open(p)
                                                    .expect("couldn't create/open file");

                                                let mut bufw = std::io::BufWriter::new(file);
                                                let sgf: String = self.goban.history.into_game_tree().into();
                                                bufw.write_all(sgf.as_bytes()).expect("couldn't write to file");
                                            },
                                            None => (),
                                        }
                                    }
                                }
                            },
                            KeyCode::W => {
                                self.engine.ownership = !self.engine.ownership;
                            }
                            _ => (),
                        }
                    },
                    _ => (),
                }
            },
            Message::StartAnalyze => {
                match self.engine_state.state() {
                    EngineStateState::Idle => {
                        let _ = self.engine_state.consume(&EngineStateInput::StartAnalyze);
                        self.engine.start_analyze();
                    },
                    EngineStateState::Analyzing => (),
                }
            },
            Message::RefreshAnalyze => {
                match self.engine_state.state() {
                    EngineStateState::Analyzing => {
                        self.engine.start_analyze();
                    },
                    EngineStateState::Idle => (),
                }
            },
            Message::StopAnalyze => {
                match self.engine_state.state() {
                    EngineStateState::Analyzing => {
                        let _ = self.engine_state.consume(&EngineStateInput::StopAnalyze);
                        self.engine.stop_analyze();
                    },
                    EngineStateState::Idle => (),
                }
            },
            Message::EngineTick(_) => {
                match self.engine.get_info() {
                    Some(info) => { self.goban.analyze_info = Some(goban::AnalyzeInfo(info)); dbg!(&self.goban.analyze_info);},
                    None => (),
                }
                let _ = self.update(Self::Message::RefreshAnalyze);
            },
            Message::EngineError => (),
            Message::OpenFile(path) => {
                let sgf = std::fs::read_to_string(path).expect("failed to load sgf");
                let game = sgf_parser::parse(sgf.as_str()).expect("failed to parse sgf");
                self.goban = Goban::default();
                self.goban.history = history::History::from(game);
            },
            Message::EngineCommand(c) => {
                match c {
                    EngineCommand::EnginePlay(t, p) => {
                        match self.engine.play(t, p) {
                            Ok(answer) => match answer {
                                libgtp::Answer::Failure(f) => {
                                    log::error!("{:?}", f);
                                    self.goban.previous_state();
                                },
                                _ => (),
                            },
                            Err(e) => {
                                log::error!("{}", e);
                                self.goban.previous_state();
                            }
                        }
                    },
                    EngineCommand::EngineUndo => self.engine.undo(),
                }
            },
            Message::Goban(_) => match self.goban.update(message) {
                Some(c) => {
                    let _ = self.update(c);
                },
                None => (),
            },
            _ => (),
        };
        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        match self.engine_state.state() {
            EngineStateState::Analyzing => {
                let timer = iced::time::every(engine_commands::TIMER_INTERVAL)
                    .map(Message::EngineTick);
                let runtime = iced_native::subscription::events().map(Message::Event);
                iced::Subscription::batch(vec![timer, runtime])
            },
            _ => iced_native::subscription::events().map(Message::Event),
        }
        
    }

    fn view(&self) -> Element<Self::Message> {
        row!(self.engine.view(), column!(self.goban.view())
            .spacing(10)
            .padding(10)
            .align_items(iced::Alignment::Center)
        )
        .spacing(10)
        .padding(10)
        .align_items(iced::Alignment::Center)
        .into()
        
    }
}

fn main() {
    scrub_log::init().unwrap();
    info!("Starting the app");
    let mut settings = Settings::default();
    settings.antialiasing = true;
    Ainalyzer::run(settings).unwrap();
}
