use druid::widget::prelude::*;
use druid::Color;
use druid::kurbo::{Line, Circle, Rect};

#[derive(Debug, Clone)]
pub struct Goban {
}


impl Widget<crate::RootState> for Goban {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut crate::RootState, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &crate::RootState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &crate::RootState, _data: &crate::RootState, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &crate::RootState, _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &crate::RootState, _env: &Env) {
        let rect = Rect::from(ctx.region().to_rect().contained_rect_with_aspect_ratio(1.0));
        let size = rect.height();
        let fill_color = Color::rgb8(219, 185, 52);

        ctx.fill(rect, &fill_color);

        let horizontal_lines: Vec<Line> = (1..20).map(|x| 
            Line::new((rect.x0 + size/20.0, rect.y0 + (x as f64 * (size/20.0))), 
                (rect.x1- size/20.0, rect.y0 + (x as f64 * (size/20.0))))).collect();
        for line in horizontal_lines {
            ctx.stroke(line, &Color::BLACK, 1.5);
        }

        let vertical_lines: Vec<Line> = (1..20).map(|x|
            Line::new((rect.x0 + (x as f64 * (size/20.0)), rect.y0 + size/20.0),
                (rect.x0 + (x as f64 * (size/20.0)), rect.y1 - size/20.0))).collect();

        for line in vertical_lines {
            ctx.stroke(line, &Color::BLACK, 1.5);
        }

        ctx.stroke(Circle::new((rect.x0 + 4.0 * size/20.0, rect.y0 + 4.0 * size/20.0), size/400.0), &Color::BLACK, size/400.0);
        ctx.stroke(Circle::new((rect.x1 - 4.0 * size/20.0, rect.y0 + 4.0 * size/20.0), size/400.0), &Color::BLACK, size/400.0);
        ctx.stroke(Circle::new((rect.x0 + 4.0 * size/20.0, rect.y1 - 4.0 * size/20.0), size/400.0), &Color::BLACK, size/400.0);
        ctx.stroke(Circle::new((rect.x1 - 4.0 * size/20.0, rect.y1 - 4.0 * size/20.0), size/400.0), &Color::BLACK, size/400.0);
        ctx.stroke(Circle::new((rect.x0 + 10.0 * size/20.0, rect.y0 + 4.0 * size/20.0), size/400.0), &Color::BLACK, size/400.0);
        ctx.stroke(Circle::new((rect.x0 + 4.0 * size/20.0, rect.y0 + 10.0 * size/20.0), size/400.0), &Color::BLACK, size/400.0);
        ctx.stroke(Circle::new((rect.x1 - 4.0 * size/20.0, rect.y0 + 10.0 * size/20.0), size/400.0), &Color::BLACK, size/400.0);
        ctx.stroke(Circle::new((rect.x0 + 10.0 * size/20.0, rect.y1 - 4.0 * size/20.0), size/400.0), &Color::BLACK, size/400.0);
        ctx.stroke(Circle::new((rect.x0 + 10.0 * size/20.0, rect.y0 + 10.0 * size/20.0), size/400.0), &Color::BLACK, size/400.0);
    }
}
