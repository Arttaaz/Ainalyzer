use druid::widget::prelude::*;
use druid::{Color, MouseButton};
use druid::kurbo::{Line, Circle, Rect};
use crate::Player;
use std::collections::HashSet;
use log::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stone {
    pub visible: bool,
    pub color: Player,
}

impl Default for Stone {
    fn default() -> Self {
        Self {
            visible: false,
            color: Player::Black,
        }
    }
}

impl Stone {


    pub fn black() -> Self {
        Self {
            visible: true,
            color: Player::Black,
        }
    }

    pub fn white() -> Self {
        Self {
            visible: true,
            color: Player::White,
        }
    }

    fn draw(&self, ctx: &mut PaintCtx, rect: &Rect, size: f64, coord: Point) {
        let color = match self.color {
            Player::Black => Color::BLACK,
            Player::White => Color::WHITE,
        };

        ctx.fill(Circle::new((rect.x0 + coord.x as f64 * size/20.0, rect.y0 + coord.y as f64 * size/20.0), size/41.8), &color);
        ctx.stroke(Circle::new((rect.x0 + coord.x as f64 * size/20.0, rect.y0 + coord.y as f64 * size/20.0), size/41.8), &Color::BLACK, 1.0);
    }

    fn hover(&self, ctx: &mut PaintCtx, rect: &Rect, size: f64, coord: Point) { 
        match self.color {
            Player::Black => ctx.fill(Circle::new((rect.x0 + coord.x as f64 * size/20.0, rect.y0 + coord.y as f64 * size/20.0), size/41.8), &Color::BLACK.with_alpha(0.7)),
            Player::White => ctx.fill(Circle::new((rect.x0 + coord.x as f64 * size/20.0, rect.y0 + coord.y as f64 * size/20.0), size/41.8), &Color::WHITE.with_alpha(0.7)),
        }
        ctx.stroke(Circle::new((rect.x0 + coord.x as f64 * size/20.0, rect.y0 + coord.y as f64 * size/20.0), size/41.8), &Color::BLACK, 1.0);
    }
}

#[derive(Debug, Clone)]
struct Group {
    stones: Vec<Point>,
    liberties: u64,
    team: Player,
}


#[derive(Debug, Clone)]
pub struct Goban {
    pub stones: Vec<Stone>, //The points coord are used as the goban coords for the stones to be placed
    pub hover: Option<(Point, Stone)>,
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
                    let pos = Point::new(pos.x as u32, pos.y as u32);
                    if pos.x > 0 && pos.y > 0 &&
                       pos.x < 20 && pos.y < 20 {
                       self.hover = match data.turn {
                            Player::Black => Some((pos, Stone::black())),
                            Player::White => Some((pos, Stone::white())),
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
                if self.hover.is_some() {
                    let (p, s) = self.hover.as_ref().unwrap().clone();
                    if mouse_event.button == MouseButton::Left && !self.stones[p.x as usize * 20 + p.y as usize].visible {
                        self.stones[p.x as usize * 20 + p.y as usize] = s;
                        self.hover = None;

                        let mut opponent_died = false;
                        let groups = self.find_groups();
                        let dead_groups = groups.iter().filter(|g| {
                            if g.liberties == 0 {
                                if g.team != data.turn {
                                    opponent_died = true;
                                }
                                true
                            } else {
                                false
                            }
                        }).collect::<Vec<_>>();

                        dead_groups.iter().for_each(|g| {
                            if opponent_died && g.team != data.turn {
                                for p in &g.stones {
                                    let i = self.coord_to_idx(*p);
                                    self.stones[i] = Stone::default();
                                    //add to captures
                                }
                            } else if !opponent_died && g.team == data.turn {
                               self.stones[p.x as usize * 20 + p.y as usize] = Stone::default(); 
                            } 
                        });
                        
                        
                        data.turn.next();
                        ctx.request_paint();
                    }
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
            let (p,s) = self.hover.as_ref().unwrap();
            s.hover(ctx, &rect, size, p.clone());
        }

        self.stones.iter().enumerate().for_each(|(i, s)| {
            if s.visible {
                s.draw(ctx, &rect, size, self.idx_to_coord(i));
            }
        });
    }

}

impl Goban {
    fn idx_to_coord(&self, i: usize) -> Point {
        Point::new(i as u32 / 20, i as u32 % 20)
    }

    fn coord_to_idx(&self, p: Point) -> usize {
        p.x as usize * 20 + p.y as usize
    }
    
    fn find_groups(&self) -> Vec<Group> {
        let mut stones = self.stones.iter().enumerate().filter_map(|(i,s)| {
            if s.visible {
                Some((i, s))
            } else {
                None
            }
        }).collect::<Vec<_>>();
        
        let mut liberties = Vec::new();
        let mut stack = Vec::new();
        let mut seen = HashSet::new();
        let mut groups = Vec::new();
        let mut group_stones = Vec::new();

        while let Some((i, s)) = stones.pop() { // take one stone from the board
            let color = s.color.clone();
            stack.push(i); // add the stone to the stack of stone to explore
            seen.insert(self.idx_to_coord(i));
            while let Some(i) = stack.pop() { // while there is stones to explore
                group_stones.push(self.idx_to_coord(i)); // add the stone to the group
                for p in self.surrounding_points(self.idx_to_coord(i)) { // for each point around us
                    if !seen.insert(p) {
                        continue;
                    }
                let s = self.stones[self.coord_to_idx(p)].clone(); // retrieve stone for color
                    if !s.visible { // if intersection is empty and we have not yet counted this liberty
                        liberties.push(p); // add to liberties
                    } else if s.color == color { // if intersection is filled, and it's the same color as us
                        stack.push(self.coord_to_idx(p));
                        stones.retain(|(x, _s)| {
                            *x != self.coord_to_idx(p)});
                    }
                } 
            }
            let group = Group {
                stones: group_stones.clone(),
                liberties: liberties.len() as u64,
                team: color,
            };
            groups.push(group);

            liberties.clear();
            seen.clear();
            group_stones.clear();
        }

        groups
    }

    fn surrounding_points(&self, p: Point) -> impl Iterator<Item = Point> {
        let x = p.x as i64;
        let y = p.y as i64;
        [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .iter()
            .filter_map(move |&(dx, dy)| {
                if (x + dx) >= 1 && x + dx < 20 as i64 && (y + dy) >= 1 && y + dy < 20 as i64 {
                    Some(Point::new((x + dx) as u32, (y + dy) as u32))
                } else {
                    None
                }
            })
    }
}








