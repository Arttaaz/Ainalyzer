use std::io::Write;
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
            data.engine.lock().expect("couldn't get engine").write("kata-analyze".as_bytes()).unwrap(); }), 0.1)
    }
}
