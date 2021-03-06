use druid::widget::prelude::*;
use druid::{Data, Color, KeyEvent, KbKey, MouseButton};
use druid::kurbo::{Line, Circle, Rect};
use crate::Player;
use std::collections::HashSet;
use std::sync::Arc;
//use log::debug;

#[derive(Debug, Clone, Data, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
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

    fn possibilty(&self, ctx: &mut PaintCtx, rect: &Rect, size: f64, coord: Point) {
        match self.color {
            Player::Black =>
                ctx.fill(Circle::new((rect.x0 + (coord.x+1) as f64 * size/20.0, rect.y0 + (coord.y+1) as f64 * size/20.0), size/70.0), &Color::BLACK.with_alpha(0.5)),
            Player::White =>
                ctx.fill(Circle::new((rect.x0 + (coord.x+1) as f64 * size/20.0, rect.y0 + (coord.y+1) as f64 * size/20.0), size/70.0), &Color::WHITE.with_alpha(0.5)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Group {
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
        ctx.request_focus();
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
                        let tmp_point = Goban::coord_to_idx(pos);
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
                if ctx.is_hot() {
                    if self.hover.is_some() {
                        let (p, s) = self.hover.as_ref().unwrap().clone();
                        if mouse_event.button == MouseButton::Left && !self.stones[Goban::coord_to_idx(p)].visible {
                            let history = Arc::make_mut(&mut data.history);
                            if history.set_variation_to_move(Goban::coord_to_idx(p)) {
                                self.next_state(history, &mut data.turn);
                            } else {
                                self.play(history, &mut data.turn, p, s);
                            }
                            ctx.request_paint();
                        }
                    }
                }
            },
            Event::Wheel(mouse_event) => {
                if mouse_event.wheel_delta.y < 0.0 {
                    let history = Arc::make_mut(&mut data.history);
                    self.previous_state(history, &mut data.turn);
                    ctx.request_paint();
                } else {
                    let history = Arc::make_mut(&mut data.history);
                    self.next_state(history, &mut data.turn);
                    ctx.request_paint();
                }
            },
            Event::KeyUp(KeyEvent {
                key: code,
                ..
            }) => {
                match code {
                    KbKey::ArrowLeft => {
                        let history = Arc::make_mut(&mut data.history);
                        self.previous_state(history, &mut data.turn);
                        ctx.request_paint();
                    },
                    KbKey::ArrowRight => {
                        let history = Arc::make_mut(&mut data.history);
                        self.next_state(history, &mut data.turn);
                        ctx.request_paint();
                    },
                    KbKey::Character(s) => {
                        match s.as_str() {
                            "o" => {
                                let open_options = druid::FileDialogOptions::new()
                                    .allowed_types(vec![druid::FileSpec::new("sgf", &["sgf"])]);
                                ctx.submit_command(druid::Command::new(druid::commands::SHOW_OPEN_PANEL, open_options, druid::Target::Auto));
                            },
                            "d" => {
                                let _history = Arc::make_mut(&mut data.history).into_game_tree();
                            },
                            "n" => {
                                if data.is_file_updated() {}
                                self.stones = vec![Stone::default(); 19*19 as usize];
                                self.hover = None;
                                self.last_move = None;
                                self.ko = None;
                            },
                            "s" => {
                                if data.path.is_some() {
                                    ctx.submit_command(druid::Command::new(druid::commands::SAVE_FILE, None, druid::Target::Auto));
                                } else {
                                    let open_options = druid::FileDialogOptions::new()
                                        .allowed_types(vec![druid::FileSpec::new("sgf", &["sgf"])]);
                                    ctx.submit_command(druid::Command::new(druid::commands::SHOW_SAVE_PANEL, open_options, druid::Target::Auto));
                                }
                            },
                            _ => (),
                        }
                    }
                    _ => (),
                }
            },
            Event::Command(s) => {
                if s.get(druid::commands::OPEN_FILE).is_some() {
                    self.stones = vec![Stone::default(); 19*19 as usize];
                    self.hover = None;
                    self.last_move = None;
                    self.ko = None;
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

    fn paint(&mut self, ctx: &mut PaintCtx, data: &crate::RootState, _env: &Env) {
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

        let variations = data.history.get_possible_moves();
        if variations.len() > 1 {
            let stone = match data.turn {
                Player::Black => Stone::black(),
                Player::White => Stone::white(),
            };
            for v in variations {
                let p = Goban::idx_to_coord(v);
                stone.possibilty(ctx, &rect, size, p);
            }
        }

        if self.hover.is_some() {
            let (p,s) = self.hover.as_ref().unwrap();
            s.hover(ctx, &rect, size, p.clone());
        }

        self.stones.iter().enumerate().for_each(|(i, s)| {
            if s.visible {
                s.draw(ctx, &rect, size, Goban::idx_to_coord(i));
            }
        });

        if self.ko.is_some() {
            let ko = self.ko.unwrap();
            ctx.stroke(Rect::from_center_size((rect.x0 + (ko.x+1) as f64 * size/20.0, rect.y0 + (ko.y+1) as f64 * size/20.0), (size/42.0, size/42.0)), &Color::BLACK, 1.5);
        }

        if self.last_move.is_some() {
            let coord = self.last_move.unwrap();
            match data.turn {
                Player::Black =>
                    ctx.stroke(
                        Circle::new((rect.x0 + (coord.x+1) as f64 * size/20.0, rect.y0 + (coord.y+1) as f64 * size/20.0), size/60.0), &Color::BLACK, 3.0),
                Player::White =>
                    ctx.stroke(
                        Circle::new((rect.x0 + (coord.x+1) as f64 * size/20.0, rect.y0 + (coord.y+1) as f64 * size/20.0), size/60.0), &Color::WHITE, 3.0),
            }
        }
    }

}

impl Goban {
    pub fn idx_to_coord(i: usize) -> Point {
        Point::new(i as u32 / 19, i as u32 % 19)
    }

    pub fn coord_to_idx(p: Point) -> usize {
        p.x as usize * 19 + p.y as usize
    }

    pub fn play(&mut self, history: &mut crate::History, turn: &mut Player, p: Point, s: Stone) {
        // check if move is in history, if it is, use next_state
        let point = Goban::coord_to_idx(p);
        self.stones[point] = s;
        match self.is_legal_move(&turn, self.last_move) {
            Ok(dead_groups) => {
                self.ko = None;
                if dead_groups.len() == 2 {
                    let mut we_died = false;
                    dead_groups.iter().for_each(|g| {
                        if g.team == *turn {
                            we_died = true;
                        } else {
                            if g.stones.len() == 1 { // we check for a ko
                                let is_ko = self.is_legal_move(match turn {
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
                    if g.team != *turn {
                        enemy_groups.push(g.clone());
                        for p in &g.stones {
                            let i = Goban::coord_to_idx(*p);
                            self.stones[i] = Stone::default();
                            //add to captures
                        }
                    }
                });
                match history.push((turn.clone(), point, enemy_groups)) {
                    Ok(_) => (),
                    Err(_) => panic!(), // show message something went wrong
                }
                self.last_move = Some(p);
                self.hover = None;
                turn.next();
            },
            Err(_) => (),
        }
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
            seen.insert(Goban::idx_to_coord(i));
            while let Some(i) = stack.pop() { // while there is stones to explore
                group_stones.push(Goban::idx_to_coord(i)); // add the stone to the group
                for p in self.surrounding_points(Goban::idx_to_coord(i)) { // for each point around us
                    if !seen.insert(p) {
                        continue;
                    }
                let s = self.stones[Goban::coord_to_idx(p)].clone(); // retrieve stone for color
                    if !s.visible { // if intersection is empty and we have not yet counted this liberty
                        liberties.push(p); // add to liberties
                    } else if s.color == color { // if intersection is filled, and it's the same color as us
                        stack.push(Goban::coord_to_idx(p));
                        stones.retain(|(x, _s)| {
                            *x != Goban::coord_to_idx(p)});
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
pub struct Move {
    pub player: Player,
    pub index: usize,
    pub groups: Vec<Group>,
}

impl From<(Player, usize, Vec<Group>)> for Move {
    fn from(m: (Player, usize, Vec<Group>)) -> Self {
        Move {
            player: m.0,
            index: m.1,
            groups: m.2,
        }
    }
}

impl Into<(Player, usize, Vec<Group>)> for Move {
    fn into(self) -> (Player, usize, Vec<Group>) {
        (self.player, self.index, self.groups)
    }
}

impl Goban {
    pub fn previous_state(&mut self, history: &mut crate::History, turn: &mut Player) {
        if let Some((previous_move, (player, played_move, dead_stones))) = history.pop() {
            self.stones[played_move] = Stone::default();
            for group in dead_stones {
                for p in &group.stones {
                    let i = Goban::coord_to_idx(*p);
                    self.stones[i] = match player {
                        Player::Black => Stone::white(),
                        Player::White => Stone::black(),
                    }
                }
            }
            turn.next();
            self.hover = None;
            self.last_move = if let Some(idx) = previous_move {
                Some(Goban::idx_to_coord(idx))
            } else {
                None
            };
        }
    }

    pub fn next_state(&mut self, history: &mut crate::History, turn: &mut Player) {
        if let Some((player, played_move, dead_stones)) = history.next() {
            self.stones[played_move] = match player {
                Player::Black => Stone::black(),
                Player::White => Stone::white(),
            };
            for group in dead_stones {
                for p in &group.stones {
                    let i = Goban::coord_to_idx(*p);
                    self.stones[i] = Stone::default();
                }
            }
            turn.next();
            self.hover = None;
            self.last_move = Some(Goban::idx_to_coord(played_move));
        }
    }
}
