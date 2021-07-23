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
                    let analyze = Arc::make_mut(&mut data.analyze_state);
                    **analyze = data.engine.lock().expect("couldn't get engine").read_info().expect("failed to parse info");
                    data.analyze_timer_token = Arc::new(Some(ctx.request_timer(std::time::Duration::from_micros(50))));
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
        .with_flex_child(Button::new("Analyze").on_click(|ctx, data: &mut crate::RootState, _| {
            let mut engine = data.engine.lock().expect("couldn't get engine");
            match engine.send_command("kata-analyze interval 50 ownership true".parse().unwrap()).unwrap() {
                libgtp::Answer::Response(_) => { 
                    ctx.submit_notification(crate::selectors::ANALYZE_TIMER_TOKEN); 
                    let analyze = Arc::make_mut(&mut data.analyze_state);
                    **analyze = engine.read_info().expect("failed to parse info I guess");
                    data.analyze_timer_token = Arc::new(Some(ctx.request_timer(std::time::Duration::from_micros(50))));
                },
                libgtp::Answer::Failure(f) => eprintln!("{}", f),
                _ => unreachable!(),
            };
        }), 0.1)
        .with_flex_child(Button::new("Stop").on_click(|_, data: &mut crate::RootState, _| {
            match data.engine.lock().expect("couldn't get engine").send_command("stop".parse().unwrap()).unwrap() {
                libgtp::Answer::Response(_) => {
                    data.analyze_timer_token = Arc::new(None);
                },
                libgtp::Answer::Failure(f) => { eprintln!("{}", f); },
                _ => unreachable!(),
            };}), 0.1)
    }
}
