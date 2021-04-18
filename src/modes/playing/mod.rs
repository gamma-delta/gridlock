mod cards;
mod cells;
mod economy;
mod trucks;

use self::{
    cards::{Card, SelectedCard},
    cells::{Cell, Instruction},
    trucks::{Treads, Truck},
};
use crate::{
    drawutils::{
        self, BOARD_ORIGIN_X, BOARD_ORIGIN_Y, CARD_PADDING, CARD_TOP_POS, CARD_WIDTH, HEX_RADIUS,
        HEX_WIDTH,
    },
    GameMode, Globals, Transition,
};
use cards::CardInstruction;

use drawutils::{TextAlign, HUD_LEFT_POS};
use economy::{Factory, Item, ItemFilter, Market, Pricer};
use hex2d::{Angle, Coordinate, Direction, Spacing, Spin};
use macroquad::prelude::{
    is_key_down, is_key_pressed, is_mouse_button_down, mouse_position, mouse_wheel, KeyCode,
    MouseButton,
};
use rand::Rng;

use std::{
    cell,
    collections::{hash_map::Entry, HashMap},
    f32::consts::TAU,
};

use super::ModeLose;

/// Tax happens every this many frames
const TAX_TIMER: u64 = 60 * 20;
/// Tax increase from colliding with a wreckage or going off the board
const TAX_COLLISION: u32 = 5;
/// Tax increase from two trucks crashing into each other
const TAX_CRASH: u32 = 10;
/// Tax increase from picking up a good when full
const TAX_OVERLOAD: u32 = 1;
/// Tax increase from bringing an empty truck to a market
const TAX_SHORTSELL: u32 = 2;
/// Tax increase from bringing the wrong thing to market
const TAX_BAD_SELL: u32 = 5;

/// Radius two trucks must be within each other to collide
const TRUCK_CRASH_RADIUS: f32 = 0.5;

/// Board-pixel distance between the center of the truck and where treads ought to be drawn
const TREAD_OFFSET: f32 = (10.0 / 64.0) * (64.0 / HEX_WIDTH);
/// How long a tread lives for
const TREAD_LIFETIME: u64 = 300;
/// Number of frames at which the tread begins to face
const TREAD_FADE_TIME: u64 = 60;

/// Max hand size
pub const HAND_SIZE: usize = 5;

pub struct ModePlaying {
    board: Board,
    player_info: PlayerInfo,
    /// How many frames this mode has been alive
    frames_elapsed: u64,
}

struct Board {
    /// Maps coordinates to cells.
    /// Any empty cells are impassable and shouldn't be driven into.
    cells: HashMap<Coordinate, Cell>,
    /// All the trucks
    trucks: Vec<Truck>,
    /// All the treads.
    /// TODO this might be a lot of things to keep track of?
    treads: Vec<Treads>,
    /// Radius of the board proper (not counting the buildings on the outsides)
    radius: usize,
}

struct PlayerInfo {
    /// How much money I have
    money: u32,
    /// Current tax rate
    tax: u32,
    /// The most amount of money I've ever had
    highscore: u32,

    /// The cards the player has in hand
    hand: Vec<Card>,
    /// The currently selected card
    selected_card: Option<SelectedCard>,
}

impl ModePlaying {
    // sHUT UP CLIPPY AAAAAAAAAAAAAAAAAAAAAAAAAAAA
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let radius = 5;
        let mut board = Board {
            cells: {
                let mut map = HashMap::new();
                for coord in Coordinate::new(0, 0).range_iter(radius as i32) {
                    map.insert(coord, Cell::Empty);
                }
                map
            },
            trucks: vec![],
            treads: vec![],
            radius,
        };

        // Generate stuff
        for idx in 0..6 {
            board.add_building((idx + 1) * 250);
        }

        let player_info = PlayerInfo {
            hand: Card::starting_hand(),
            highscore: 0,
            money: 100,
            tax: 0,
            selected_card: None,
        };

        Self {
            board,
            player_info,
            frames_elapsed: 0,
        }
    }

    pub fn update(&mut self, globals: &mut Globals) -> Transition {
        let (dmoney, dtax) = self.board.update(self.frames_elapsed, globals);
        self.player_info.money += dmoney;
        self.player_info.tax += dtax;
        self.player_info.highscore += dmoney;

        if self.frames_elapsed % TAX_TIMER == 0 && self.frames_elapsed != 0 && self.apply_tax() {
            return Transition::Swap(GameMode::Lose(ModeLose::new(self.player_info.highscore)));
        }

        // Input
        match &mut self.player_info.selected_card {
            None => {
                use macroquad::prelude::*;
                if is_mouse_button_pressed(MouseButton::Left) {
                    // Check if i'm in a correct card zone
                    let (mouse_x, mouse_y) = mouse_position();
                    let card_idx = mouse_x / (CARD_WIDTH + CARD_PADDING);
                    if card_idx > 1.0 && mouse_y >= CARD_TOP_POS {
                        let card_idx = card_idx as usize - 1;
                        if card_idx < self.player_info.hand.len() {
                            // noice we select this
                            let card = self.player_info.hand[card_idx].clone();
                            let rotation = match &card {
                                Card::Truck { .. } => 2,
                                Card::Instruction(CardInstruction::Rotate) => 4,
                                Card::Instruction(CardInstruction::Direct) => 2,
                                Card::Instruction(CardInstruction::Shunt) => 2,
                                Card::Cleanup => 0,
                            };

                            self.player_info.selected_card = Some(SelectedCard {
                                original_idx: Some(card_idx),
                                card,
                                rotation,
                                position: mouse_position(),
                            });
                        } else if card_idx == HAND_SIZE {
                            // draw a new card
                            if self.apply_tax() {
                                return Transition::Swap(GameMode::Lose(ModeLose::new(
                                    self.player_info.highscore,
                                )));
                            }
                        }
                    } else {
                        let coord = Coordinate::from_pixel(
                            mouse_x - BOARD_ORIGIN_X,
                            mouse_y - BOARD_ORIGIN_Y,
                            Spacing::PointyTop(HEX_RADIUS),
                        );
                        let cell = self.board.cells.get(&coord);
                        if let Some(Cell::Instruction(..)) = cell {
                            // there must be a better way to do this
                            let instr = match self.board.cells.insert(coord, Cell::Empty) {
                                Some(Cell::Instruction(i)) => i,
                                _ => unreachable!(),
                            };
                            let (card_instr, rotation) = match &instr {
                                Instruction::Rotate(angle) => {
                                    (CardInstruction::Rotate, angle.to_int())
                                }
                                Instruction::Direct(dir) => (CardInstruction::Direct, dir.to_int()),
                                Instruction::Shunt(dir) => (CardInstruction::Shunt, dir.to_int()),
                            };
                            self.player_info.selected_card = Some(SelectedCard {
                                card: Card::Instruction(card_instr),
                                rotation,
                                original_idx: None,
                                position: mouse_position(),
                            });
                        }
                    }
                }
            }
            Some(selected) => {
                if !is_mouse_button_down(MouseButton::Left) {
                    enum CardStatus {
                        Place,
                        ReturnToHand,
                        Discard,
                    };

                    // check if i'm in the grid
                    let board_x = mouse_position().0 - BOARD_ORIGIN_X;
                    let board_y = mouse_position().1 - BOARD_ORIGIN_Y;
                    let coord =
                        Coordinate::from_pixel(board_x, board_y, Spacing::PointyTop(HEX_RADIUS));

                    let card_status =
                        if coord.distance(Coordinate::new(0, 0)) <= self.board.radius as i32 {
                            // put it there!
                            match &selected.card {
                                Card::Instruction(card_instr) => {
                                    if matches!(
                                        self.board.cells.get(&coord),
                                        Some(Cell::Empty) | Some(Cell::Instruction(_))
                                    ) {
                                        // nice
                                        let instr = card_instr.to_instruction(selected.rotation);
                                        self.board.cells.insert(coord, Cell::Instruction(instr));
                                        CardStatus::Place
                                    } else {
                                        CardStatus::ReturnToHand
                                    }
                                }
                                Card::Truck { cargo } => {
                                    self.board.trucks.push(Truck::from_rot(
                                        coord,
                                        cargo.clone(),
                                        selected.rotation,
                                    ));
                                    CardStatus::Place
                                }
                                Card::Cleanup => {
                                    if self.board.cells.contains_key(&coord) {
                                        self.board.cells.insert(coord, Cell::Empty);
                                        CardStatus::Place
                                    } else {
                                        CardStatus::ReturnToHand
                                    }
                                }
                            }
                        } else {
                            let mouse_x = mouse_position().0;
                            let card_idx = mouse_x / (CARD_WIDTH + CARD_PADDING);
                            // We "successfully" throw the card away if we're in the trash zone
                            if card_idx < 1.0 {
                                CardStatus::Discard
                            } else {
                                CardStatus::ReturnToHand
                            }
                        };

                    match card_status {
                        CardStatus::Place => {
                            if selected.card.cost() <= self.player_info.money {
                                // Nice we play the card
                                // Remove the card
                                if let Some(original_idx) = selected.original_idx {
                                    self.player_info.hand.remove(original_idx);
                                    // Pay money if we had an original idx, meaning it came from the hand
                                    self.player_info.money -= selected.card.cost();
                                }
                            }
                        }
                        CardStatus::Discard => {
                            // Remove the card
                            if let Some(original_idx) = selected.original_idx {
                                self.player_info.hand.remove(original_idx);
                            }
                        }
                        CardStatus::ReturnToHand => {
                            // do nothing
                        }
                    }
                    // in any case stop selecting
                    self.player_info.selected_card = None;
                } else {
                    // mouse_wheel: up is positive, down is negative.
                    let scroll = mouse_wheel().1;
                    if scroll > 0.0 || is_key_pressed(KeyCode::A) {
                        selected.rotation -= 1;
                    } else if scroll < 0.0 || is_key_pressed(KeyCode::D) {
                        selected.rotation += 1;
                    }
                }
            }
        }

        self.frames_elapsed += 1;

        Transition::None
    }

    pub fn draw(&self, globals: &Globals) {
        use macroquad::prelude::*;

        clear_background(Color::from_rgba(250, 252, 255, 255));

        self.board.draw(globals);
        self.player_info.draw(self.frames_elapsed, globals);
    }

    // return `true` to quit
    fn apply_tax(&mut self) -> bool {
        match self.player_info.money.checked_sub(self.player_info.tax) {
            Some(it) => self.player_info.money = it,
            None => {
                // oh no we ran out of money :(
                return true;
            }
        }

        if self.player_info.hand.len() < HAND_SIZE {
            self.player_info.hand.push(Card::generate());
        }
        self.player_info.tax += 1;
        false
    }
}

impl Board {
    /// Update the board and trucks.
    ///
    /// Return `(delta_money, delta_tax)`
    fn update(&mut self, frames_elapsed: u64, globals: &mut Globals) -> (u32, u32) {
        let mut money = 0;
        let mut tax = 0;

        // Update all my truccs and remove the collided ones
        let cells = &mut self.cells;
        let treads = &mut self.treads;
        self.trucks
            .drain_filter(|truck| {
                truck.move_progress += truck.speed;

                // Where the truck is going
                let target = truck.position + truck.facing;
                // Where the center of the truck is
                let realpos = truck.get_hex();

                if truck.move_progress > 0.5 {
                    // we crossed a hex, no longer immune
                    truck.out_of_bounds_immunity = false;
                }

                // Add treads
                if frames_elapsed % 3 == 0 {
                    let truck_pos = truck.get_xy();
                    let (dy, dx) = (truck.facing.to_radians_pointy::<f32>() - TAU / 4.0).sin_cos();
                    let pos = (
                        truck_pos.0 + dx * TREAD_OFFSET,
                        truck_pos.1 + dy * TREAD_OFFSET,
                    );
                    treads.push(Treads {
                        pos,
                        facing: truck.facing,
                        lifetime: TREAD_LIFETIME,
                    });
                }

                if !Board::is_passable(cells, realpos, truck.out_of_bounds_immunity) {
                    // oh no, clobber this position into wreckage
                    match cells.get(&truck.position) {
                        Some(Cell::Factory(..)) | Some(Cell::Market(..)) => {
                            cells.remove(&truck.position);
                        }
                        Some(..) => {
                            cells.insert(truck.position, Cell::Wreckage);
                        }
                        _ => {}
                    }

                    tax += TAX_COLLISION;
                    // delt trucc
                    return true;
                }

                if truck.move_progress >= 1.0 {
                    // We're on the center of the next coord
                    truck.position = target;
                    truck.move_progress = 0.0;

                    // Take a special action?
                    let cell = cells.entry(truck.position);
                    if let Entry::Occupied(mut occupied) = cell {
                        match occupied.get_mut() {
                            Cell::Factory(factory) => {
                                if truck.cargo.is_some() {
                                    // uh-oh
                                    tax += TAX_OVERLOAD;
                                }
                                truck.cargo = Some(factory.product.clone());
                                factory.stock -= 1;
                                if factory.stock == 0 {
                                    // clear the factory
                                    occupied.remove();
                                    // and it's ok to drive over empty for now
                                    truck.out_of_bounds_immunity = true;
                                }
                                truck.facing = truck.facing + Angle::Back;
                            }
                            Cell::Market(market) => {
                                match &truck.cargo {
                                    None => {
                                        // uh-oh
                                        tax += TAX_SHORTSELL;
                                    }
                                    Some(item) => {
                                        if market.request.matches(item) {
                                            // noice
                                            money += market.prices.sample();
                                            market.demand -= 1;
                                            if market.demand == 0 {
                                                occupied.remove();
                                                truck.out_of_bounds_immunity = true;
                                            }
                                        } else {
                                            // oh no
                                            tax += TAX_BAD_SELL;
                                        }
                                        truck.cargo = None;
                                    }
                                }
                                truck.facing = truck.facing + Angle::Back;
                            }
                            Cell::Instruction(instruction) => match *instruction {
                                Instruction::Rotate(rot) => {
                                    truck.facing = truck.facing + rot;
                                }
                                Instruction::Direct(dir) => {
                                    truck.facing = dir;
                                }
                                Instruction::Shunt(shunt) => {
                                    let target = truck.position + shunt;
                                    if !Board::is_passable(
                                        cells,
                                        target,
                                        truck.out_of_bounds_immunity,
                                    ) {
                                        // oh no, clobber this position into wreckage
                                        if cells.contains_key(&truck.position) {
                                            cells.insert(truck.position, Cell::Wreckage);
                                        }
                                        tax += TAX_COLLISION;
                                        // delt trucc
                                        return true;
                                    } else {
                                        truck.position = target;
                                    }
                                }
                            },
                            Cell::Empty => {}
                            Cell::Wreckage => {
                                // :HOW:
                                println!("A truck has fallen into the wreckage in Error City!");
                                return true;
                            }
                        }
                    }
                }

                // otherwise keep
                false
            })
            .for_each(drop); // get rid of bad ones

        // Update treads
        self.treads
            .drain_filter(|tread| {
                if tread.lifetime == 0 {
                    // drop it
                    true
                } else {
                    tread.lifetime -= 1;
                    false
                }
            })
            .for_each(drop);

        // Check for collisions
        let mut collided_truck_idxes = Vec::new();
        for (idx, truck) in self.trucks.iter().enumerate() {
            let (x, y) = truck.get_xy();
            // prevent collisions with self
            for other in self.trucks.iter().skip(idx + 1) {
                let (ox, oy) = other.get_xy();
                if (x - ox).powi(2) + (y - oy).powi(2) < TRUCK_CRASH_RADIUS.powi(2) {
                    // oeuf
                    tax += TAX_CRASH;
                    collided_truck_idxes.push(idx);
                    let hex = truck.get_hex();
                    if self.cells.contains_key(&hex) {
                        self.cells.insert(hex, Cell::Wreckage);
                    }
                    // the other piece of wreckage will be inserted by the other truck.
                }
            }
        }
        for idx in (0..self.trucks.len()).rev() {
            if collided_truck_idxes.contains(&idx) {
                self.trucks.remove(idx);
            }
        }

        // Update prices and add markets
        let mut building_count = 0;
        for (_coord, cell) in self.cells.iter_mut() {
            if let Cell::Market(m) = cell {
                m.prices.timestep();
                building_count += 1;
            } else if let Cell::Factory(..) = cell {
                building_count += 1;
            }
        }
        if building_count < 6 + (1 + frames_elapsed / 3600) * self.trucks.len() as u64
            && frames_elapsed % 60 == 0
            && rand::thread_rng().gen_bool(0.2)
        {
            self.add_building(frames_elapsed);
        }

        (money, tax)
    }

    fn draw(&self, globals: &Globals) {
        use macroquad::prelude::*;

        // Cells that gotta be drawn *after* trucks
        // (&cell, coord) pairs
        let mut toppers = Vec::new();

        for (&coord, cell) in self.cells.iter() {
            match cell {
                Cell::Factory(_) | Cell::Market(_) => toppers.push((coord, cell)),
                _ => cell.draw(coord, globals),
            }
        }

        // Draw treads
        for tread in self.treads.iter() {
            let opacity = if tread.lifetime > TREAD_FADE_TIME {
                1.0
            } else {
                tread.lifetime as f32 / TREAD_FADE_TIME as f32
            };
            let cx = tread.pos.0 * HEX_WIDTH + BOARD_ORIGIN_X;
            let cy = tread.pos.1 * HEX_WIDTH + BOARD_ORIGIN_Y;

            draw_texture_ex(
                globals.assets.textures.treads,
                cx - 1.0,
                cy - 17.0,
                Color::new(1.0, 1.0, 1.0, opacity),
                DrawTextureParams {
                    rotation: tread.facing.to_radians_pointy::<f32>() - TAU / 4.0,
                    pivot: Some(vec2(cx, cy)),
                    ..Default::default()
                },
            );
        }

        for truck in self.trucks.iter() {
            truck.draw(globals);
        }
        for (coord, cell) in toppers {
            cell.draw(coord, globals);
        }
    }

    /// Check if the given hex can be driven through
    fn is_passable(
        cells: &HashMap<Coordinate, Cell>,
        coord: Coordinate,
        out_of_bounds_immunity: bool,
    ) -> bool {
        let target = cells.get(&coord);
        match target {
            None => out_of_bounds_immunity,
            Some(Cell::Wreckage) => false,
            _ => true,
        }
    }

    /// Add a new market or factory, accounting for current buildings.
    fn add_building(&mut self, frames_elapsed: u64) {
        let mut rng = rand::thread_rng();

        // Count the number of markets. If there are markets without a factory for them, add the factory
        // This maps items to bitmaps. Bit 1 = factory, bit 2 = market.
        let mut item_statuses = HashMap::<_, u8>::new();
        for (_coord, cell) in self.cells.iter() {
            match cell {
                Cell::Factory(f) => {
                    *item_statuses.entry(&f.product).or_default() |= 0b01;
                }
                Cell::Market(Market {
                    request: ItemFilter::Specific(item),
                    ..
                }) => {
                    *item_statuses.entry(&item).or_default() |= 0b10;
                }
                _ => {}
            }
        }

        let lacking_factory = item_statuses
            .into_iter()
            .filter(|(_item, bitmask)| *bitmask == 0b10)
            .collect::<Vec<_>>();
        let new_building = if lacking_factory.is_empty() {
            // Nothing is lacking, make up something totally random
            if rng.gen_bool(0.6) {
                Cell::Factory(Factory::generate(Item::sample(), frames_elapsed))
            } else {
                Cell::Market(Market::generate(
                    if rng.gen_bool(0.2) {
                        ItemFilter::Any
                    } else {
                        ItemFilter::Specific(Item::sample())
                    },
                    frames_elapsed,
                ))
            }
        } else {
            Cell::Factory(Factory::generate(
                lacking_factory[rng.gen_range(0..lacking_factory.len())]
                    .0
                    .to_owned(),
                frames_elapsed,
            ))
        };

        // Insert it somewhere, hopefully
        let mut canidates = Coordinate::new(0, 0)
            .ring_iter(self.radius as i32 + 1, Spin::CW(Direction::XY))
            .collect::<Vec<_>>();
        // shut
        #[allow(clippy::map_entry)]
        loop {
            let end = match canidates.len() {
                0 => break,
                it => it,
            };
            let coord = canidates.remove(rng.gen_range(0..end));
            if !self.cells.contains_key(&coord) {
                self.cells.insert(coord, new_building);
                break;
            }
        }
    }
}

impl PlayerInfo {
    fn draw(&self, frames_elapsed: u64, globals: &Globals) {
        use macroquad::prelude::*;

        draw_texture(
            globals.assets.textures.delete_card,
            CARD_PADDING,
            CARD_TOP_POS,
            WHITE,
        );
        for (idx, card) in self.hand.iter().enumerate() {
            match &self.selected_card {
                Some(sel) if sel.original_idx == Some(idx) => {
                    // skip this card
                }
                _ => {
                    let x = (idx + 1) as f32 * (CARD_WIDTH + CARD_PADDING);
                    card.draw(x, CARD_TOP_POS, self.money, globals);
                }
            }
        }
        if let Some(sel) = &self.selected_card {
            let (cx, cy) = mouse_position();
            sel.draw(cx, cy, globals);
        }

        drawutils::text(
            globals,
            &format!("Money: ${}", self.money),
            18,
            HUD_LEFT_POS,
            CARD_TOP_POS + 20.0,
            TextAlign::Left,
        );
        drawutils::text_color(
            globals,
            &format!("Tax: ${}", self.tax),
            18,
            HUD_LEFT_POS,
            CARD_TOP_POS + 40.0,
            TextAlign::Left,
            RED,
        );
        drawutils::text(
            globals,
            &format!("Timer: {}", TAX_TIMER - frames_elapsed % TAX_TIMER),
            18,
            HUD_LEFT_POS,
            CARD_TOP_POS + 60.0,
            TextAlign::Left,
        );
        drawutils::text(
            globals,
            &format!("Score: ${}", self.highscore),
            18,
            HUD_LEFT_POS,
            CARD_TOP_POS + 90.0,
            TextAlign::Left,
        );
    }
}
