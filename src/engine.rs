use crate::engine_commands::*;
use std::sync::{Arc, Mutex};

use iced::widget::button;
use iced::Element;


#[derive(Clone)]
pub struct Engine {
    controller: Arc<Mutex<libgtp::Controller>>,
    pub ownership: bool,
}

impl Engine {
    pub fn new() -> Self {
        let mut controller = libgtp::Controller::new("./KataGo/katago", &["gtp", "-model", "./KataGo/model.bin.gz", "-config", "./KataGo/default_gtp.cfg"]);
        controller.send_command(COMMAND_RULES_JAPANESE.clone()).unwrap();
        controller.send_command(COMMAND_KOMI.clone()).unwrap();
        controller.send_command(COMMAND_CLEARBOARD.clone()).unwrap();
        Self {
            controller: Arc::new(Mutex::new(controller)),
            ownership: false,
        }
    }

    #[allow(dead_code)]
    pub fn restart_engine(&mut self) {
        let mut controller = libgtp::Controller::new("./KataGo/katago", &["gtp", "-model", "./KataGo/model.bin.gz", "-config", "./KataGo/default_gtp.cfg"]);
        controller.send_command(COMMAND_RULES_JAPANESE.clone()).unwrap();
        controller.send_command(COMMAND_KOMI.clone()).unwrap();
        controller.send_command(COMMAND_CLEARBOARD.clone()).unwrap();
        self.controller = Arc::new(Mutex::new(controller));
    }

    pub fn view<'a>(&'a self) -> Element<'a, crate::Message> {
        let analyze = button("Start analyze")
            .on_press(crate::Message::StartAnalyze);
        let stop = button("Stop analyze")
            .on_press(crate::Message::StopAnalyze);

        iced::widget::Column::new()
            .push(iced::widget::row!(analyze, stop).spacing(20))
            .height(iced::Length::FillPortion(1))
            .align_items(iced::Alignment::Center)
            .into()
    }

    pub fn start_analyze(&self) -> libgtp::Answer {
        let mut engine = self.controller.lock().expect("could not get engine");
        match self.ownership {
            true => engine.send_command(COMMAND_ANALYZE_OWNERSHIP.clone()).unwrap(),
            false => engine.send_command(COMMAND_ANALYZE.clone()).unwrap(),

        }
    }
    
    pub fn stop_analyze(&self) -> libgtp::Answer {
        let mut engine = self.controller.lock().expect("could not get engine");
        engine.send_command(COMMAND_STOP.clone()).unwrap()
    }

    pub fn get_info(&self) -> Option<libgtp::Info> {
        let engine = self.controller.lock().expect("could not get engine");
        match engine.read_info() {
            Ok(info) => info,
            Err(e) => {
                log::error!("engine controller: {:?}", e);
                None
            }
        }
    }

    pub fn play(&mut self, turn: crate::Player, p: crate::goban::Point, winrate: Option<(u64, f32)>) -> Result<libgtp::Answer, std::io::Error> {
        let mut engine = self.controller.lock().expect("could not get engine");
        let answer = engine.send_command(format!("play {} {}", turn, p).as_str().parse().unwrap())?;
        Ok(answer)
    }

    pub fn undo(&mut self) {
        let mut engine = self.controller.lock().expect("could not get engine");
        engine.send_command(COMMAND_UNDO.clone()).unwrap();
    }
}
