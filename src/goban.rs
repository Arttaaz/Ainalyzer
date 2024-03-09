use crate::Player;
use crate::history::*;
use crate::Message;
use crate::GobanEvent;
use std::collections::HashSet;
use libgtp::model::Info;
use iced::{widget::canvas, Rectangle, Element};
use iced::widget::canvas::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let x = if self.x > 8 {
            self.x + 1
        } else {
            self.x
        };

        write!(f, "{}{}", ((x as u8 + 65) as char).to_uppercase(), self.y+1)
    }
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
        }
    }

    pub fn as_coord_tuple(&self) -> (u8, u8) {
        (self.x as u8, self.y as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub fn new(color: Player) -> Self {
        Self {
            visible: true,
            color
        }
    }
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


    fn hover(&self, rect: &Rectangle, coord: &Point) -> (Path, iced::Color) {
        let size = rect.height;
        let size_stone = size/20.0;
        let radius = size/41.8;
        let color = match self.color {
            Player::Black => iced::Color::new(0.0, 0.0, 0.0, 0.7),
            Player::White => iced::Color::new(1.0, 1.0, 1.0, 0.7),
        };
        (Path::circle(iced::Point {x: rect.x + (coord.x + 1) as f32 * size_stone ,y:  rect.y + (coord.y + 1) as f32 * size_stone}, radius),
         color.into())
    }

    fn possibilty(&self, rect: &Rectangle, coord: Point) -> (Path, iced::Color) {
        let size = rect.height;
        let size_stone = size/20.0;
        let radius = size/70.0;
        let color = match self.color {
            Player::Black => iced::Color::new(0.0, 0.0, 0.0, 0.7),
            Player::White => iced::Color::new(1.0, 1.0, 1.0, 0.7),
        };
        //(Path::circle((rect.x0 + (coord.x+1) as f64 * size/20.0, rect.y0 + (coord.y+1) as f64 * size/20.0), size/70.0), &Color::BLACK.with_alpha(0.5)),
        (Path::circle(iced::Point {x: rect.x + (coord.x + 1) as f32 * size_stone ,y:  rect.y + (coord.y + 1) as f32 * size_stone}, radius),
         color.into())
    }
}

#[derive(Debug, Clone)]
pub struct AnalyzeInfo(pub Info);

impl AnalyzeInfo {
    #[allow(dead_code)]
    pub fn max_winrate(&self) -> f32 {
        self.0.explored_moves.iter()
            .filter(|x| x.coord.to_tuple().is_some())
            .map(|x| x.winrate * 100.0)
            .max_by_key(|x| (x * 1000.0) as u64).unwrap()
    }

    fn winrate_of(&self, turn: Player, p: Point) -> Option<f32> {
        if let Some(mov) = self.0.explored_moves.iter()
                            .find(|x| x.coord.to_tuple().is_some() && x.coord.to_tuple().unwrap() == p.as_coord_tuple()) {
            Some(if turn == Player::Black {mov.winrate} else {1.0 - mov.winrate} * 100.0)
        } else {
            None
        }
    }

    fn draw(&self, frame: &mut canvas::Frame, rect: &Rectangle, size: f32, player: Player) {
        self.0.ownership.iter().enumerate().for_each(|(point, ownership)| {
            let mut color = if ownership.is_sign_positive() {
                match player {
                    Player::Black => iced::Color::BLACK,
                    Player::White => iced::Color::WHITE,
                }
            } else {
                match player {
                    Player::Black => iced::Color::WHITE,
                    Player::White => iced::Color::BLACK,
                }
            };
            color.a = ownership.abs() * 0.7;
            let point = Point::new(point as u32 % 19, (360 - point as u32) / 19);
            frame.fill(&Path::rectangle(iced::Point { x: rect.x + (point.x+1) as f32 * size/20.0 - size/40.0, y: rect.y + (point.y+1) as f32 * size/20.0 - size/40.0}, iced::Size { width: size/20.0, height: size/20.0 }), color);
        });
        let max_winrate = self.0.explored_moves.iter()
            .filter(|x| x.coord.to_tuple().is_some())
            .map(|x| ((x.winrate * 1000.0) as u64, (x.coord.to_tuple().unwrap())))
            .max_by_key(|x| x.0).unwrap();
        let max_visits = self.0.explored_moves.iter()
            .filter(|x| x.coord.to_tuple().is_some())
            .map(|x| (x.visits, (x.coord.to_tuple().unwrap())))
            .max_by_key(|x| x.0).unwrap();

        let gradiant_pos = canvas::gradient::Position::Relative {
            top_left: iced::Point { x: rect.x + (max_winrate.1.0) as f32 * size/20.0 - size/40.0, y: rect.y + (max_winrate.1.1) as f32 * size/20.0 - size/40.0 },
            size: iced::Size { width: size/20.0, height: size/20.0 },
            start: canvas::gradient::Location::TopLeft,
            end: canvas::gradient::Location::BottomRight,
        };
        let gradiant = canvas::Gradient::linear(gradiant_pos)
            .add_stop(0.0, iced::color!(0, 0, 0, 0.0))
            .add_stop(0.5, iced::Color::from_rgba(0.0, 1.0, 0.0, 0.6))
            .add_stop(1.0, iced::color!(0, 0, 0, 0.0))
            .build().expect("failed to create gradiant");
        frame.fill(&Path::circle(iced::Point { x: rect.x + (max_winrate.1.0) as f32 * size/20.0, y: rect.y + (max_winrate.1.1) as f32 * size/20.0 }, size/40.0), gradiant);

        let gradiant_pos = canvas::gradient::Position::Relative {
            top_left: iced::Point { x: rect.x + (max_visits.1.0) as f32 * size/20.0 - size/40.0, y: rect.y + (max_visits.1.1) as f32 * size/20.0 - size/40.0 },
            size: iced::Size { width: size/20.0, height: size/20.0 },
            start: canvas::gradient::Location::TopRight,
            end: canvas::gradient::Location::BottomLeft,
        };
        let gradiant = canvas::Gradient::linear(gradiant_pos)
            .add_stop(0.0, iced::color!(0, 0, 0, 0.0))
            .add_stop(0.5, iced::color!(7, 109, 252, 0.6))
            .add_stop(1.0, iced::color!(0, 0, 0, 0.0))
            .build().expect("failed to create gradiant");
        frame.fill(&Path::circle(iced::Point { x: rect.x + (max_visits.1.0) as f32 * size/20.0, y: rect.y + (max_visits.1.1) as f32 * size/20.0 }, size/40.0), gradiant);

        self.0.explored_moves.iter().for_each(|move_info| {
            let point = move_info.coord.to_tuple();
            if let Some((x, y)) = point {
                if ((x, y) != max_winrate.1) && ((x, y) != max_visits.1) {
                    let gradiant_pos = canvas::gradient::Position::Relative {
                        top_left: iced::Point { x: rect.x + (x) as f32 * size/20.0 - size/40.0, y: rect.y + (y) as f32 * size/20.0 - size/40.0 },
                        size: iced::Size { width: size/20.0, height: size/20.0 },
                        start: canvas::gradient::Location::TopRight,
                        end: canvas::gradient::Location::BottomLeft,
                    };
                    let gradiant = canvas::Gradient::linear(gradiant_pos)
                        .add_stop(0.0, iced::color!(0, 0, 0, 0.0))
                        .add_stop(0.5, iced::color!(255, 0, 0, 0.6))
                        .add_stop(1.0, iced::color!(0, 0, 0, 0.0))
                        .build().expect("failed to create gradiant");
                    frame.fill(&Path::circle(iced::Point { x: rect.x + (x) as f32 * size/20.0, y: rect.y + (y) as f32 * size/20.0 }, size/40.0), gradiant);
                }

                let mut text = canvas::Text::default();
                text.content = format!("{:^4.1}%", move_info.winrate * 100.0);
                text.size = size/69.0;
                text.horizontal_alignment = iced::alignment::Horizontal::Center;
                text.position = iced::Point { x: rect.x + (x as f32) * size/20.0, y: rect.y + (y as f32 - 0.32) * size/20.0 };
                frame.fill_text(text);
                let mut text = canvas::Text::default();
                text.content = format_num::format_num!("^.2s", move_info.visits as f64);
                text.size = size/69.0;
                text.horizontal_alignment = iced::alignment::Horizontal::Center;
                text.position = iced::Point { x: rect.x + (x as f32) * size/20.0, y: rect.y + (y as f32) * size/20.0 };
                frame.fill_text(text);
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    stones: Vec<Point>,
    liberties: u64,
    team: Player,
}

#[derive(Debug, Default, Clone)]
pub struct GobanState {
    pub hover: Option<Point>,
}

#[derive(Debug, Clone)]
pub struct Goban {
    pub stones: Vec<Stone>, //The points coord are used as the goban coords for the stones to be placed
    pub last_move: Option<Point>,
    pub ko: Option<Point>,
    pub current_move_number: u16,
    pub total_move_number: u16,
    pub history: History,
    pub turn: Player,
    pub analyze_info: Option<AnalyzeInfo>,
}

impl Default for Goban {
    fn default() -> Self {
        Self {
            stones: vec![Stone::default(); 19*19 as usize],
            last_move: None,
            ko: None,
            current_move_number: 0,
            total_move_number: 0,
            history: History::default(),
            turn: Player::Black,
            analyze_info: None,
        }
    }
}

impl Goban {
    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        canvas(self)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    fn stones_to_path(&self, rect: &Rectangle) -> Vec<(Path, iced::Color)> {
        let size = rect.height;
        let size_stone = size/20.0;
        let radius = size/41.8;
        self.stones.iter().enumerate().filter(|(_, s)| s.visible).map(|(i, s)| {
            let coord = Goban::idx_to_coord(i);
            (Path::circle(iced::Point {x: rect.x + (coord.x + 1) as f32 * size_stone ,y:  rect.y + (coord.y + 1) as f32 * size_stone}, radius),
             s.color.into())
        }).collect()
    }

    fn hover_update(&self, state: &mut GobanState, rect: &Rectangle, position: &iced::Point) {
        let size = if rect.height > rect.width {
            rect.width
        } else {
            rect.height
        };
        let point = iced::Point::new((rect.width - size)/2.0, (rect.height - size) / 2.0);
        let square = Rectangle::new(point, iced::Size { width: size, height: size });
        let pos = Point::new((((position.x - square.x) / (size/20.0)-0.0).round() - 1.0) as u32,
                             (((position.y - size/40.0 - square.y) / (size/20.0)+0.45).round() - 1.0) as u32);

        if (pos.x < 19 && pos.y < 19) && (position.x >= square.x && position.y >= square.y) {
            let tmp_point = Goban::coord_to_idx(pos);
            if !self.stones[tmp_point].visible {
                let mut stones = self.stones.clone();
                stones[tmp_point] = Stone::new(self.turn);

                if Goban::is_legal_move(&stones, &self.turn, self.last_move).is_ok() {
                    state.hover = match self.turn {
                        Player::Black => Some(pos),
                        Player::White => Some(pos),
                    };
                }
            } else {
                state.hover = None;
            }
        } else {
            state.hover = None;
        }
    }
}

impl Goban {
    pub fn update(&mut self, message: crate::Message) -> Option<Message> {
        let analyze_info = self.analyze_info.clone();
        self.analyze_info = None;
        match message {
            Message::Goban(event) => match event {
                GobanEvent::Play(p, s) => {
                    if self.history.set_variation_to_move(Goban::coord_to_idx(p)) {
                        self.next_state();
                    } else {
                        self.play(p, s);
                    }
                    let winrate = if analyze_info.is_some() {
                        let mut pred_turn = self.turn;
                        pred_turn.next();
                        if let Some(w) = analyze_info.unwrap().winrate_of(pred_turn, p) {
                            Some((self.current_move_number as u64, w))
                        } else {
                            None
                        }
                    }
                    else {
                        None
                    };
                    return Some(Message::EngineCommand(crate::EngineCommand::EnginePlay(match self.turn { Player::Black => Player::White, Player::White => Player::Black}, p, winrate)))
                },
                GobanEvent::NextState => {
                    if self.next_state() {
                        return Some(Message::EngineCommand(crate::EngineCommand::EnginePlay(match self.turn { Player::Black => Player::White, Player::White => Player::Black}, self.last_move.unwrap(), None)))
                    }
                },
                GobanEvent::PreviousState => {
                    if self.previous_state() {
                        return Some(Message::EngineCommand(crate::EngineCommand::EngineUndo))
                    }
                },
            },
            _ => (),
        }
        None
    }
}

impl<'a> canvas::Program<Message> for Goban {
    type State = GobanState;

    fn draw(
            &self,
            state: &GobanState,
            _theme: &iced::theme::Theme,
            bounds: Rectangle,
            _cursor_position: canvas::Cursor,
        ) -> Vec<canvas::Geometry> {
        
        let mut frame = canvas::Frame::new(bounds.size());

        // create goban square bounded by the layout
        let square_size = if bounds.height > bounds.width {
            bounds.width
        } else {
            bounds.height
        };
        //let point = iced::Point::new(bounds.center_x() - square_size/2.0, bounds.center_y() - square_size/2.0);
        let point = iced::Point::new((bounds.width - square_size)/2.0, (bounds.height - square_size) / 2.0);
        let square = Path::rectangle(point, iced::Size { width: square_size, height: square_size });

        // fill Goban background
        frame.fill(&square, iced::Color::from_rgb8(219, 185, 52));

        // add the lines
        let spacing = square_size / 20.0;
        let x_right = point.x + square_size - spacing;
        let y_down = point.y + square_size - spacing;

        let mut lines: Vec<Path> = (1..20).map(|x|
            Path::line(iced::Point { x: point.x + spacing, y: point.y + (x as f32 * spacing)}, iced::Point {x: x_right, y: point.y + (x as f32 * spacing)})).collect();
        
        let mut vertical_lines: Vec<Path> = (1..20).map(|x|
            Path::line(iced::Point { x: point.x + (x as f32 * spacing), y: point.y + spacing}, iced::Point {x: point.x + (x as f32 * spacing), y: y_down})).collect();
        
        lines.append(&mut vertical_lines);

        let stroke = canvas::Stroke::default()
            .with_color(iced::Color::BLACK)
            .with_width(2.0);

        for line in lines {
            frame.stroke(&line, stroke.clone());
        }

        let small_radius = square_size/250.0;
        frame.fill(&Path::circle(iced::Point::new(point.x + 4.0 * spacing, point.y + 4.0 * spacing), small_radius), iced::Color::BLACK);
        frame.fill(&Path::circle(iced::Point::new(x_right - 3.0 * spacing, point.y + 4.0 * spacing), small_radius), iced::Color::BLACK);
        frame.fill(&Path::circle(iced::Point::new(point.x + 4.0 * spacing, y_down - 3.0 * spacing), small_radius), iced::Color::BLACK);
        frame.fill(&Path::circle(iced::Point::new(x_right - 3.0 * spacing, y_down - 3.0 * spacing), small_radius), iced::Color::BLACK);
        frame.fill(&Path::circle(iced::Point::new(point.x + 10.0 * spacing, point.y + 4.0 * spacing), small_radius), iced::Color::BLACK);
        frame.fill(&Path::circle(iced::Point::new(point.x + 4.0 * spacing, point.y + 10.0 * spacing), small_radius), iced::Color::BLACK);
        frame.fill(&Path::circle(iced::Point::new(x_right - 3.0 * spacing, point.y + 10.0 * spacing), small_radius), iced::Color::BLACK);
        frame.fill(&Path::circle(iced::Point::new(point.x + 10.0 * spacing, y_down - 3.0 * spacing), small_radius), iced::Color::BLACK);
        frame.fill(&Path::circle(iced::Point::new(point.x + 10.0 * spacing, point.y + 10.0 * spacing), small_radius), iced::Color::BLACK);

        let rectangle = Rectangle::new(point, iced::Size::new(square_size, square_size));

        let variations = self.history.get_possible_moves();
        if variations.len() > 1 {
            let stone = match self.turn {
                Player::Black => Stone::black(),
                Player::White => Stone::white(),
            };
            for v in variations {
                let p = Goban::idx_to_coord(v);
                let (path, color) = stone.possibilty(&rectangle, p);
                frame.fill(&path, color);
            }
        }

        if state.hover.is_some() {
            let p = state.hover.as_ref().unwrap();
            let (path, color) = Stone::new(self.turn).hover(&rectangle, p);
            frame.fill(&path, color);
            let border = canvas::Stroke::default()
                .with_color(iced::Color::BLACK)
                .with_width(1.0);
            frame.stroke(&path, border);
        }

        // add visible stones
        for (path, color) in self.stones_to_path(&rectangle).iter() {
            frame.fill(path, *color);
            let border = canvas::Stroke::default()
                .with_color(iced::Color::BLACK)
                .with_width(1.0);
            frame.stroke(path, border);
        }

        if self.analyze_info.is_some() {
            self.analyze_info.as_ref().unwrap().draw(&mut frame, &rectangle, rectangle.height, self.turn);
        }

        if self.ko.is_some() {
            let ko = self.ko.unwrap();
            let size = rectangle.height;
            let border = canvas::Stroke::default()
                .with_color(iced::Color::BLACK)
                .with_width(3.0);
            frame.stroke(&Path::rectangle(iced::Point { x: rectangle.x + (ko.x+1) as f32 * size/20.0 - size/84.0, y: rectangle.y + (ko.y+1) as f32 * size/20.0 - size/84.0}, iced::Size { width: size/42.0, height: size/42.0 }), border);
        }

        if self.last_move.is_some() {
            let coord = self.last_move.unwrap();
            let size = rectangle.height;
            let size_stone = size/20.0;
            let radius = size/60.0;
            let color = match self.turn {
                Player::Black => iced::Color::new(0.0, 0.0, 0.0, 1.0),
                Player::White => iced::Color::new(1.0, 1.0, 1.0, 1.0),
            }; 
            let border = canvas::Stroke::default()
                .with_color(color)
                .with_width(3.0);
            frame.stroke(&Path::circle(iced::Point {x: rectangle.x + (coord.x + 1) as f32 * size_stone ,y:  rectangle.y + (coord.y + 1) as f32 * size_stone}, radius), border);
        }

        vec![frame.into_geometry()]
    }

    fn update(
            &self,
            state: &mut GobanState,
            event: canvas::Event,
            bounds: Rectangle,
            cursor: canvas::Cursor,
        ) -> (canvas::event::Status, Option<Message>) {
        if cursor.is_over(&bounds) {
            match event {
                canvas::Event::Mouse(event) => match event {
                    iced::mouse::Event::CursorMoved { position: _ } => self.hover_update(state, &bounds, &cursor.position_in(&bounds).unwrap()),
                    iced::mouse::Event::ButtonReleased(b) => match b {
                        iced::mouse::Button::Left => {
                            if state.hover.is_some() {
                                let p = state.hover.as_ref().unwrap().clone();
                                let s = Stone::new(self.turn);
                                state.hover = None;
                                return (canvas::event::Status::Captured, Some(Message::Goban(crate::GobanEvent::Play(p, s))))
                            }
                        },
                        iced::mouse::Button::Right => (),
                        _ => (),
                    },
                    iced::mouse::Event::WheelScrolled { delta } => {
                        match delta {
                            iced::mouse::ScrollDelta::Lines { x: _, y } => {
                                if y < 0.0 {
                                    return (canvas::event::Status::Captured, Some(Message::Goban(crate::GobanEvent::NextState)))
                                } else if y > 0.0 {
                                    return (canvas::event::Status::Captured, Some(Message::Goban(crate::GobanEvent::PreviousState)))
                                }
                            },
                            iced::mouse::ScrollDelta::Pixels { x: _, y } => {
                                if y < 0.0 {
                                    return (canvas::event::Status::Captured, Some(Message::Goban(crate::GobanEvent::NextState)))
                                } else if y > 0.0 {
                                    return (canvas::event::Status::Captured, Some(Message::Goban(crate::GobanEvent::PreviousState)))
                                }
                            },
                        }
                    }
                    _ => (),
                },
                canvas::Event::Keyboard(ev) => match ev {
                    iced::keyboard::Event::KeyReleased { key_code, modifiers: _ } => {
                        match key_code {
                            iced::keyboard::KeyCode::Left | iced::keyboard::KeyCode::Up => {
                                return (canvas::event::Status::Captured, Some(Message::Goban(crate::GobanEvent::PreviousState)))
                            },
                            iced::keyboard::KeyCode::Right | iced::keyboard::KeyCode::Down => {
                                return (canvas::event::Status::Captured, Some(Message::Goban(crate::GobanEvent::NextState)))
                            },
                            _ => (),
                        }
                    },
                    _ => (),
                },
                _ => (),
            }
        }
        (canvas::event::Status::Ignored, None)
    }
}

impl Goban {
    pub fn idx_to_coord(i: usize) -> Point {
        Point::new(i as u32 / 19, i as u32 % 19)
    }

    pub fn coord_to_idx(p: Point) -> usize {
        p.x as usize * 19 + p.y as usize
    }

    pub fn play(&mut self, p: Point, s: Stone) {
        // check if move is in history, if it is, use next_state
        let point = Goban::coord_to_idx(p);
        self.stones[point] = s;
        let (dead_groups, _, _) = Goban::find_dead_groups(&self.stones, &self.turn);
        self.ko = None;
        if dead_groups.len() == 2 {
            let mut we_died = false;
            dead_groups.iter().for_each(|g| {
                if g.team == self.turn {
                    we_died = true;
                } else {
                    if g.stones.len() == 1 { // we check for a ko
                        let is_ko = Goban::is_legal_move(&self.stones, match self.turn {
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
            if g.team != self.turn {
                enemy_groups.push(g.clone());
                for p in &g.stones {
                    let i = Goban::coord_to_idx(*p);
                    self.stones[i] = Stone::default();
                    //add to captures
                }
            }
        });
        match self.history.push((self.turn.clone(), point, enemy_groups)) {
            Ok(_) => (),
            Err(_) => panic!(), // show message something went wrong
        }
        self.last_move = Some(p);
        self.turn.next();
        self.current_move_number += 1;
    }


    fn find_groups(stones_vec: &Vec<Stone>) -> Vec<Group> {
        let mut stones = stones_vec.iter().enumerate().filter_map(|(i,s)| {
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
                for p in Goban::surrounding_points(Goban::idx_to_coord(i)) { // for each point around us
                    if !seen.insert(p) {
                        continue;
                    }
                let s = stones_vec[Goban::coord_to_idx(p)].clone(); // retrieve stone for color
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

    fn surrounding_points(p: Point) -> impl Iterator<Item = Point> {
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

    fn find_dead_groups(stones: &Vec<Stone>, current_turn: &Player) -> (Vec<Group>, bool, bool) { //bool is true if an opponent's group died, second one if we died
        let mut opponent_died = false;
        let mut we_died = false;
        let groups = Goban::find_groups(stones);
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

    pub fn is_legal_move(stones: &Vec<Stone>, current_turn: &Player, last_move: Option<Point>) -> Result<Vec<Group>, ()> {
        let (dead_groups, opponent_died, we_died) = Goban::find_dead_groups(stones, current_turn);
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
    pub fn previous_state(&mut self) -> bool {
        if let Some((previous_move, (player, played_move, dead_stones))) = self.history.pop() {
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
            self.turn.next();
            self.last_move = if let Some(idx) = previous_move {
                Some(Goban::idx_to_coord(idx))
            } else {
                None
            };
            self.current_move_number -= 1;

            true
        } else {
            false
        }
    }

    pub fn next_state(&mut self) -> bool {
        if let Some((player, played_move, dead_stones)) = self.history.next() {
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
            self.turn.next();
            self.last_move = Some(Goban::idx_to_coord(played_move));
            self.current_move_number += 1;

            true
        } else {
            false
        }
    }
}
