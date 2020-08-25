use druid::widget::prelude::*;
use druid::{Data, Color, KbKey, MouseButton};
use druid::kurbo::{Line, Circle, Rect};
use crate::Player;
use std::collections::HashSet;
use std::sync::Arc;
use log::debug;

#[derive(Debug, Clone, Data, Copy, PartialEq, Eq, Hash)]
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

        ctx.fill(Circle::new((rect.x0 + (coord.x+1) as f64 * size/20.0, rect.y0 + (coord.y+1) as f64 * size/20.0), size/41.8), &color);
        ctx.stroke(Circle::new((rect.x0 + (coord.x+1) as f64 * size/20.0, rect.y0 + (coord.y+1) as f64 * size/20.0), size/41.8), &Color::BLACK, 1.0);
    }

    fn hover(&self, ctx: &mut PaintCtx, rect: &Rect, size: f64, coord: Point) {
        match self.color {
            Player::Black =>
                ctx.fill(Circle::new((rect.x0 + (coord.x+1) as f64 * size/20.0, rect.y0 + (coord.y+1) as f64 * size/20.0), size/41.8), &Color::BLACK.with_alpha(0.7)),
            Player::White =>
                ctx.fill(Circle::new((rect.x0 + (coord.x+1) as f64 * size/20.0, rect.y0 + (coord.y+1) as f64 * size/20.0), size/41.8), &Color::WHITE.with_alpha(0.7)),
        }
        ctx.stroke(Circle::new((rect.x0 + (coord.x+1) as f64 * size/20.0, rect.y0 + (coord.y+1) as f64 * size/20.0), size/41.8), &Color::BLACK, 1.0);
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
    pub last_move: Option<Point>,
    pub ko: Option<Point>,
}

impl Default for Goban {
    fn default() -> Self {
        Self {
            stones: vec![Stone::default(); 19*19 as usize],
            hover: None,
            last_move: None,
            ko: None,
        }
    }
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
                    pos.x = ((pos.x - rect.x0) / (size/20.0)).round() - 1.0;
                    pos.y = ((pos.y - rect.y0) / (size/20.0)).round() - 1.0;
                    let pos = Point::new(pos.x as u32, pos.y as u32);
                    if pos.x < 19 && pos.y < 19 {
                        let tmp_point = self.coord_to_idx(pos);
                        if !self.stones[tmp_point].visible {
                            self.stones[tmp_point] = match data.turn {
                                Player::Black => Stone::black(),
                                Player::White => Stone::white(),
                            };
                            if self.is_legal_move(&data.turn, self.last_move).is_ok() {
                                self.hover = match data.turn {
                                    Player::Black => Some((pos, Stone::black())),
                                    Player::White => Some((pos, Stone::white())),
                                };
                            }
                            self.stones[tmp_point] = Stone::default();
                        } else {
                            self.hover = None;
                        }
                    } else {
                        self.hover = None;
                    }
                } else {
                    self.hover = None;
                }
                ctx.request_paint();
            },
            Event::MouseUp(mouse_event) => {
                if ctx.is_hot(){
                    if self.hover.is_some() {
                        let (p, s) = self.hover.as_ref().unwrap().clone();
                        if mouse_event.button == MouseButton::Left && !self.stones[self.coord_to_idx(p)].visible {
                            let point = self.coord_to_idx(p);
                            self.stones[point] = s;
                            match self.is_legal_move(&data.turn, self.last_move) {
                                Ok(dead_groups) => {
                                    self.ko = None;
                                    if dead_groups.len() == 2 {
                                        let mut we_died = false;
                                        dead_groups.iter().for_each(|g| {
                                            if g.team == data.turn {
                                                we_died = true;
                                            } else {
                                                if g.stones.len() == 1 { // we check for a ko
                                                    let is_ko = self.is_legal_move(match data.turn {
                                                        Player::Black => &Player::White,
                                                        Player::White => &Player::Black,
                                                    }, Some(p)).is_err();
                                                    if is_ko {
                                                        self.ko = Some(g.stones[0]);
                                                    }
                                                }
                                            }
                                        });
                                        if !we_died {
                                            self.ko = None;
                                        }
                                    }
                                    let mut enemy_groups = Vec::new();
                                    dead_groups.iter().for_each(|g| {
                                        if g.team != data.turn {
                                            enemy_groups.push(g.clone());
                                            for p in &g.stones {
                                                let i = self.coord_to_idx(*p);
                                                self.stones[i] = Stone::default();
                                                //add to captures
                                            }
                                        }
                                    });
                                    Arc::make_mut(&mut data.history).push((point, enemy_groups));
                                    self.last_move = Some(p);
                                    self.hover = None;
                                    data.turn.next();
                                },
                                Err(_) => (),
                            }

                            ctx.request_paint();
                        }
                    }
                }
            },
            Event::Wheel(mouse_event) => {
                if mouse_event.wheel_delta.y < 0.0 {
                    self.previous_state(ctx, data);
                } else {
                    self.next_state(ctx, data);
                }
            },
            Event::KeyUp(key_event) => { //apparently broken
                debug!("HEY DUDE");
                match &key_event.key {
                    KbKey::ArrowLeft => self.previous_state(ctx, data),
                    KbKey::ArrowRight => self.next_state(ctx, data),
                    _ => (),
                }
            },
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &crate::RootState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &crate::RootState, _data: &crate::RootState, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &crate::RootState, _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &crate::RootState, _env: &Env) {
        let rect = Rect::from(ctx.region().bounding_box().contained_rect_with_aspect_ratio(1.0));

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

        if self.ko.is_some() {
            let ko = self.ko.unwrap();
            ctx.stroke(Rect::from_center_size((rect.x0 + (ko.x+1) as f64 * size/20.0, rect.y0 + (ko.y+1) as f64 * size/20.0), (size/42.0, size/42.0)), &Color::BLACK, 1.5);
        }
    }

}

impl Goban {
    fn idx_to_coord(&self, i: usize) -> Point {
        Point::new(i as u32 / 19, i as u32 % 19)
    }

    fn coord_to_idx(&self, p: Point) -> usize {
        p.x as usize * 19 + p.y as usize
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
                if (x + dx) >= 0 && x + dx < 19 as i64 && (y + dy) >= 0 && y + dy < 19 as i64 {
                    Some(Point::new((x + dx) as u32, (y + dy) as u32))
                } else {
                    None
                }
            })
    }

    fn find_dead_groups(&self, current_turn: &Player) -> (Vec<Group>, bool, bool) { //bool is true if an opponent's group died, second one if we died
        let mut opponent_died = false;
        let mut we_died = false;
        let groups = self.find_groups();
        let dead_groups = groups.into_iter().filter(|g| {
            if g.liberties == 0 {
                if g.team != *current_turn {
                    opponent_died = true;
                } else if g.team == *current_turn {
                    we_died = true;
                }
                true
            } else {
                false
            }
        }).collect::<Vec<_>>();

        (dead_groups, opponent_died, we_died)
    }

    fn is_legal_move(&self, current_turn: &Player, last_move: Option<Point>) -> Result<Vec<Group>, ()> {
        let (dead_groups, opponent_died, we_died) = self.find_dead_groups(current_turn);
        let mut illegal = false;
        let dead_groups = dead_groups.into_iter().filter(|group| {
            if !opponent_died && group.team == *current_turn {
                illegal = true;
                false
            } else if we_died && opponent_died && group.team != *current_turn {
                if group.stones.len() == 1 && last_move.unwrap() == group.stones[0] { //if 2 groups died we can safely unwrap last_move
                    illegal = true;
                    false
                } else {
                    true
                }
            } else {
                true
            }
        }).collect::<Vec<Group>>();

        if illegal {
            Err(())
        } else {
            Ok(dead_groups)
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameHistory {
    history: Vec<(usize, Vec<Group>)>,
    current_index: usize,
}

impl Default for GameHistory {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            current_index: 0,
        }
    }
}

impl GameHistory {
    fn push(&mut self, elem: (usize, Vec<Group>)) {
        if self.current_index + 1 < self.history.len() {
            let _ = self.history.split_off(self.current_index);
        }
        self.history.push(elem);
        self.current_index += 1;
    }

    fn pop(&mut self) -> Option<(usize, Vec<Group>)> {
        if self.current_index as isize - 1 >= 0 {
            self.current_index -= 1;
            Some(self.history[self.current_index].clone())
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<(Player, usize, Vec<Group>)> {
        if self.current_index == self.history.len() {
            None
        } else {
            let player = match self.current_index % 2 {
                0 => Player::Black,
                1 => Player::White,
                _ => unreachable!(),
            };
            self.current_index += 1;
            let (idx, dead_stones) = self.history[self.current_index-1].clone();
            Some((player, idx, dead_stones))
        }
    }
}

impl Goban {
    fn previous_state(&mut self, ctx: &mut EventCtx, data: &mut crate::RootState) {
        if let Some((played_move, dead_stones)) = Arc::make_mut(&mut data.history).pop() {
            let player = self.stones[played_move].color;
            self.stones[played_move] = Stone::default();
            for group in dead_stones {
                for p in &group.stones {
                    let i = self.coord_to_idx(*p);
                    self.stones[i] = match player {
                        Player::Black => Stone::white(),
                        Player::White => Stone::black(),
                    }
                }
            }
            data.turn.next();
            self.hover = None;
            ctx.request_paint();
        }
    }

    fn next_state(&mut self, ctx: &mut EventCtx, data: &mut crate::RootState) {
        if let Some((player, played_move, dead_stones)) = Arc::make_mut(&mut data.history).next() {
            self.stones[played_move] = match player {
                Player::Black => Stone::black(),
                Player::White => Stone::white(),
            };
            for group in dead_stones {
                for p in &group.stones {
                    let i = self.coord_to_idx(*p);
                    self.stones[i] = Stone::default();
                }
            }
            data.turn.next();
            self.hover = None;
            ctx.request_paint();
        }
    }
}
