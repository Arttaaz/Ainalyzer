#[macro_use] extern crate lazy_static;

use std::io::Write;
use iced::keyboard::key;
use log::info;
use iced::{Element, Settings, Subscription, Task};
use iced::{Event, keyboard::Key};
use iced::widget::{column, row};
use native_dialog::FileDialog;

mod engine;
use engine::Engine;

mod goban;
use goban::Goban;

mod history;
mod engine_commands;

mod winrate_plot;
use winrate_plot::WinratePlot;

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

#[derive(Debug, Clone, Copy)]
pub enum GobanEvent {
    Play(goban::Point, goban::Stone),
    PreviousState,
    NextState,
}

#[derive(Debug, Clone)]
pub enum EngineCommand {
    EnginePlay(Player, goban::Point, Option<(u64, f32)>),
    EngineUndo,
}

#[derive(Debug, Clone)]
pub enum Message {
    Event(iced::Event),
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
    winrate_plot: WinratePlot,
    opened_file: Option<std::path::PathBuf>,
    file_updated: bool,
}

impl Ainalyzer {
    fn new() -> (Self, Task<Message>) {
        (Self {
            engine: Engine::new(),
            engine_state: rust_fsm::StateMachine::new(),
            goban: Goban::default(),
            winrate_plot: WinratePlot::new(),
            opened_file: None,
            file_updated: true,
        }, Task::none())
    }

    fn title(&self) -> String {
        let mut title = match &self.opened_file {
            Some(path) => format!("AInalyzer - {}", path.file_name().unwrap().to_str().unwrap().to_owned()),
            None => String::from("AInalyzer"),
        };
        if !self.file_updated {
            title.push('*');
        }
        title
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(event) => {
                match event {
                    Event::Keyboard(iced::keyboard::Event::KeyReleased{ key, modifiers, ..}) => {
                        let key = key.as_ref();
                        if modifiers.is_empty() {
                            match key {
                                Key::Character(c) => {
                                    if c == "w" {
                                        self.engine.ownership = !self.engine.ownership;
                                    }
                                },
                                Key::Named(name) => {
                                    if name == key::Named::Space {
                                        match self.engine_state.state() {
                                            EngineStateState::Idle => {
                                                let _ = self.update(Message::StartAnalyze);
                                            },
                                            EngineStateState::Analyzing => {
                                                let _ = self.update(Message::StopAnalyze);
                                            },
                                        }
                                    }
                                }
                                _ => (),
                            }
                        } else if modifiers.control() {
                            match key {
                                Key::Character(c) => {
                                    match c {
                                        "o" => {
                                            return Task::perform(async move {
                                                FileDialog::new().show_open_single_file().expect("Open dialog failed")
                                            }, |message| {
                                                match message {
                                                    Some(m) => Message::OpenFile(m),
                                                    None => Message::DialogCancel,
                                                }
                                            })
                                        },
                                        "n" => {
                                            self.goban = Goban::default();
                                            self.opened_file = None;
                                            self.file_updated = true;
                                        }
                                        "s" => {
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
                                                    self.file_updated = true;
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
                                                            self.file_updated = true;
                                                        },
                                                        None => (),
                                                    }
                                                }
                                            }
                                        },
                                        _ => (),
                                    }
                                },
                                _ => (),
                            }
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
                    Some(info) => {
                        self.goban.analyze_info = Some(goban::AnalyzeInfo(info));
                        let winrate = self.goban.analyze_info.as_ref().unwrap().max_winrate();
                        let winrate = if self.goban.turn == Player::Black {
                            winrate
                        } else {
                            100.0 - winrate
                        };
                        self.winrate_plot.update_plot((self.goban.current_move_number as u64, winrate));
                    },
                    None => (),
                }
                let _ = self.update(Message::RefreshAnalyze);
            },
            Message::EngineError => (),
            Message::OpenFile(path) => {
                let sgf = std::fs::read_to_string(path.clone()).expect("failed to load sgf");
                let game = sgf_parser::parse(sgf.as_str()).expect("failed to parse sgf");
                self.opened_file = Some(path);
                self.goban = Goban::default();
                self.goban.history = history::History::from(game);
                self.file_updated = true;
            },
            Message::EngineCommand(c) => {
                match c {
                    EngineCommand::EnginePlay(t, p, w) => {
                        match self.engine.play(t, p, w) {
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
            Message::Goban(e) => {
                match &e {
                    GobanEvent::Play(_, _) => {
                        self.file_updated = false;
                    },
                    _ => (),
                }

                match self.goban.update(message) {
                    Some(c) => {
                        let _ = self.update(c);
                    },
                    None => (),
                }
            },
            _ => (),
        };
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.engine_state.state() {
            EngineStateState::Analyzing => {
                let timer = iced::time::every(engine_commands::TIMER_INTERVAL)
                    .map(Message::EngineTick);
                let runtime = iced::event::listen().map(Message::Event);
                iced::Subscription::batch(vec![timer, runtime])
            },
            _ => iced::event::listen().map(Message::Event),
        }
        
    }

    fn view(&self) -> Element<'_, Message> {
        let left_column = column!(
            iced_aksel::Chart::new(&self.winrate_plot.state)
                .plot_data(&self.winrate_plot, "x_id", "y_id"),
            iced::widget::space::vertical(),
            self.engine.view(),
            iced::widget::space::vertical())
                .width(iced::Length::FillPortion(1))
                .height(iced::Length::FillPortion(6));

        row!(
            left_column,
            column!(self.goban.view())
                .spacing(0)
                .padding(30)
                .width(iced::Length::FillPortion(2))
                .align_x(iced::Alignment::Center)
        )
        .spacing(0)
        .padding(0)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .align_y(iced::Alignment::Center)
        .into()
    }
}

fn main() {
    scrub_log::init().unwrap();
    info!("Starting the app");
    let mut settings = Settings::default();
    settings.antialiasing = true;
    iced::application(Ainalyzer::new, Ainalyzer::update, Ainalyzer::view)
        .subscription(Ainalyzer::subscription)
        .title(Ainalyzer::title)
        .theme(|_: &Ainalyzer| {iced::Theme::Dark})
        .run().expect("Failed to run the application");
}
