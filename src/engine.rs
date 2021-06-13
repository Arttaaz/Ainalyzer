use druid::widget::prelude::*;
use druid::{ Data, Lens, Widget,};
use druid::widget::Button;
use druid::widget::Flex;

#[derive(Clone, Data, Lens)]
pub struct Engine {
}


impl Widget<crate::RootState> for Engine {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut crate::RootState, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &crate::RootState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &crate::RootState, _data: &crate::RootState, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &crate::RootState, _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, _ctx: &mut PaintCtx, _data: &crate::RootState, _env: &Env) {
    }
}

impl Engine {
    pub fn build_engine_tab() -> impl Widget<crate::RootState> {
        Flex::row()
        .with_flex_child(Button::new("Analyze").on_click(|_, data: &mut crate::RootState, _| {
            match data.engine.lock().expect("couldn't get engine").send_command("kata-analyze".parse().unwrap()).unwrap() {
                libgtp::Answer::Response(_) => (),
                libgtp::Answer::Failure(f) => eprintln!("{}", f),
            };
            data.analyze_state = data.engine.lock().expect("couldn't get engine").read_info();
        }), 0.1)
        .with_flex_child(Button::new("Stop").on_click(|_, data: &mut crate::RootState, _| {
            match data.engine.lock().expect("couldn't get engine").send_command("stop".parse().unwrap()).unwrap() {
                libgtp::Answer::Response(r) => r.to_string(),
                libgtp::Answer::Failure(f) => { eprintln!("{}", f); String::new()},
            };}), 0.1)
    }
}
