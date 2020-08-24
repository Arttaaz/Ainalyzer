use druid::widget::prelude::*;
use druid::{Color, MouseButton, Point};
use druid::kurbo::{Line, Circle, Rect};
use crate::Player;

#[derive(Debug, Clone)]
pub struct Stone {
    pub coord: Point,
    pub color: Color,
}

impl PartialEq for Stone {
    fn eq(&self, oth: &Stone) -> bool {
        if self.coord == oth.coord {
            true
        } else {
            false
        }
    }
}

impl Stone {
    pub fn black(coord: Point) -> Self {
        Self {
            coord,
            color: Color::BLACK,
        }
    }

    pub fn white(coord: Point) -> Self {
        Self {
            coord,
            color: Color::WHITE,
        }
    }

    fn draw(&self, ctx: &mut PaintCtx, rect: &Rect, size: f64) {
        ctx.fill(Circle::new((rect.x0 + self.coord.x * size/20.0, rect.y0 + self.coord.y * size/20.0), size/41.8), &self.color);
        ctx.stroke(Circle::new((rect.x0 + self.coord.x * size/20.0, rect.y0 + self.coord.y * size/20.0), size/41.8), &Color::BLACK, 1.0);
    }

    fn hover(&self, ctx: &mut PaintCtx, rect: &Rect, size: f64) {
        if self.color.as_rgba_u32() == Color::BLACK.as_rgba_u32() {
            ctx.fill(Circle::new((rect.x0 + self.coord.x * size/20.0, rect.y0 + self.coord.y * size/20.0), size/41.8), &Color::BLACK.with_alpha(0.7));
        } else if self.color.as_rgba_u32() == Color::WHITE.as_rgba_u32() {
            ctx.fill(Circle::new((rect.x0 + self.coord.x * size/20.0, rect.y0 + self.coord.y * size/20.0), size/41.8), &Color::WHITE.with_alpha(0.7));
        }
        ctx.stroke(Circle::new((rect.x0 + self.coord.x * size/20.0, rect.y0 + self.coord.y * size/20.0), size/41.8), &Color::BLACK, 1.0);
    }
}

#[derive(Debug, Clone)]
pub struct Goban {
    pub stones: Vec<Stone>, //The points coord are used as the goban coords for the stones to be placed
    pub hover: Option<Stone>,
}


impl Widget<crate::RootState> for Goban {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut crate::RootState, _env: &Env) {
        match event {
            Event::MouseMove(mouse_event) => {
                if ctx.is_hot() {
                    let size = ctx.size();
                    let rect = Rect::from_origin_size((0.0,0.0), size).contained_rect_with_aspect_ratio(1.0);
                    let size = rect.height();
                    let mut pos = mouse_event.pos;
                    pos.x = ((pos.x - rect.x0) / (size/20.0)).round();
                    pos.y = ((pos.y - rect.y0) / (size/20.0)).round();
                    if pos.x > 0.0 && pos.y > 0.0 &&
                       pos.x < 20.0 && pos.y < 20.0 {
                       self.hover = match data.turn {
                            Player::Black => Some(Stone::black(pos)),
                            Player::White => Some(Stone::white(pos)),
                        };
                    } else {
                        self.hover = None;
                    }
                } else {
                    self.hover = None;
                }
                ctx.request_paint();
            },
            Event::MouseUp(mouse_event) => {
                if mouse_event.button == MouseButton::Left && self.hover.is_some() && !self.stones.contains(self.hover.as_ref().unwrap()) {
                    self.stones.push(self.hover.as_ref().unwrap().clone());
                    self.hover = None;
                    data.turn.next();
                    ctx.request_paint();
                }
            }
            _ => (),
        }
    }

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

        if self.hover.is_some() {
            self.hover.as_ref().unwrap().hover(ctx, &rect, size);
        }

        self.stones.iter().for_each(|p| {
            p.draw(ctx, &rect, size);
        });
    }
}
