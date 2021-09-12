use crate::engine_commands::*;
use std::sync::Arc;
use druid::widget::prelude::*;
use druid::{ Data, Lens, Widget,};
use druid::widget::Button;
use druid::widget::Flex;

#[derive(Clone, Data, Lens)]
pub struct Engine {
}


impl Widget<crate::RootState> for Engine {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut crate::RootState, _env: &Env) {
        match event {
            Event::Command(c) => {
                if c.get(crate::selectors::ANALYZE_TIMER_TOKEN).is_some() {
                    let mut analyze = data.analyze_info.lock().unwrap();
                    *analyze = data.engine.lock().expect("couldn't get engine").read_info().expect("failed to parse info");
                    data.analyze_timer_token = Arc::new(Some(ctx.request_timer(TIMER_INTERVAL.clone())));
                }
            },
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &crate::RootState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &crate::RootState, _data: &crate::RootState, _env: &Env) {
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &crate::RootState, _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, _ctx: &mut PaintCtx, _data: &crate::RootState, _env: &Env) {
    }
}

impl Engine {
    pub fn build_engine_tab() -> impl Widget<crate::RootState> {
        Flex::row()
        .with_flex_child(Engine {}, 0.1)
        .with_flex_child(Button::new("Analyze").on_click(|ctx, data: &mut crate::RootState, _| {
            let mut engine = data.engine.lock().expect("couldn't get engine");
            match engine.send_command(COMMAND_ANALYZE.clone()).unwrap() {
                libgtp::Answer::Response(_) => {
                    let mut state = data.engine_state.lock().unwrap();
                    let _ = state.consume(&crate::EngineStateInput::StartAnalyze);
                    let mut analyze = data.analyze_info.lock().unwrap();
                    *analyze = engine.read_info().expect("failed to parse info I guess");
                    data.analyze_timer_token = Arc::new(Some(ctx.request_timer(TIMER_INTERVAL.clone())));
                },
                libgtp::Answer::Failure(f) => eprintln!("{}", f),
                _ => unreachable!(),
            };
        }), 1.5)
        .with_flex_child(Button::new("Stop").on_click(|_, data: &mut crate::RootState, _| {
            match data.engine.lock().expect("couldn't get engine").send_command(COMMAND_STOP.clone()).unwrap() {
                libgtp::Answer::Response(_) => {
                    let mut state = data.engine_state.lock().unwrap();
                    let _ = state.consume(&crate::EngineStateInput::StopAnalyze);
                    data.analyze_timer_token = Arc::new(None);
                },
                libgtp::Answer::Failure(f) => { eprintln!("{}", f); },
                _ => unreachable!(),
            };}), 1.0)
    }

    pub fn engine_startup() -> libgtp::Controller {
        let mut controller = libgtp::Controller::new("./KataGo/katago", &["gtp", "-model", "./KataGo/model.bin.gz", "-config", "./KataGo/default_gtp.cfg"]);
        controller.send_command(COMMAND_RULES_JAPANESE.clone()).unwrap();
        controller.send_command(COMMAND_KOMI.clone()).unwrap();
        controller.send_command(COMMAND_CLEARBOARD.clone()).unwrap();
        controller
    }
}
