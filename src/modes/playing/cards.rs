use super::{
    cells::{Cell, Instruction},
    economy::Item,
};
use crate::{drawutils, Globals};
use crate::{
    drawutils::{CARD_HEIGHT, CARD_WIDTH},
    modes::playing::Truck,
};

use drawutils::{HEX_HEIGHT, HEX_WIDTH};
use hex2d::{Angle, Coordinate, Direction};
use macroquad::prelude::{draw_texture, WHITE};
use rand::Rng;

/// A card held in hand.
#[derive(Clone)]
pub enum Card {
    Truck { cargo: Option<Item> },
    Instruction(CardInstruction),
    Cleanup,
}

impl Card {
    /// Make the starting hand of cards
    pub fn starting_hand() -> Vec<Self> {
        vec![
            Card::Truck { cargo: None },
            Card::Truck { cargo: None },
            Card::Instruction(CardInstruction::Direct),
            Card::Instruction(CardInstruction::Shunt),
            Card::Cleanup,
        ]
    }

    /// Generate a random card.
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();

        if rng.gen_bool(0.5) {
            // Make an instruction
            Card::Instruction(CardInstruction::sample())
        } else if rng.gen_bool(0.3) {
            Card::Cleanup
        } else {
            // Trucc
            let cargo = if rng.gen_bool(0.8) {
                None
            } else {
                Some(Item::sample())
            };
            Card::Truck { cargo }
        }
    }

    /// Get the cost of this card
    pub fn cost(&self) -> u32 {
        match self {
            Card::Truck { cargo } => 50 + if cargo.is_some() { 20 } else { 0 },
            Card::Instruction(_) => 20,
            Card::Cleanup => 10,
        }
    }

    /// Draw this card. `x, y` are the corner.
    pub fn draw(&self, x: f32, y: f32, current_money: u32, globals: &Globals) {
        use macroquad::prelude::*;

        draw_texture(globals.assets.textures.card, x, y, WHITE);

        let title = match self {
            Card::Truck { .. } => "Truck",
            Card::Instruction(instr) => match instr {
                CardInstruction::Direct => "Direct",
                CardInstruction::Rotate => "Rotate",
                CardInstruction::Shunt => "Shunt",
            },
            Card::Cleanup => "Cleanup",
        };

        drawutils::center_text(globals, &title, 18, x + 49.5, y + 15.0);
        drawutils::center_text_color(
            globals,
            format!("${}", self.cost()).as_str(),
            18,
            x + 49.5,
            y + 125.0,
            if self.cost() > current_money {
                RED
            } else {
                BLACK
            },
        );

        match self {
            Card::Truck { cargo } => {
                let to_draw = Truck {
                    cargo: cargo.to_owned(),
                    speed: 0.0,
                    facing: Direction::XY,
                    position: Coordinate::new(0, 0),
                    move_progress: 0.0,
                    out_of_bounds_immunity: false,
                };
                to_draw.draw_absolute(x + 50.0, y + 70.0, globals);
            }
            Card::Instruction(instr) => {
                let cell = Cell::Instruction(match instr {
                    CardInstruction::Direct => Instruction::Direct(Direction::XY),
                    CardInstruction::Rotate => Instruction::Rotate(Angle::LeftBack),
                    CardInstruction::Shunt => Instruction::Shunt(Direction::XY),
                });
                cell.draw_absolute(
                    x + 50.0 - HEX_WIDTH / 2.0,
                    y + 70.0 - HEX_HEIGHT / 2.0,
                    globals,
                );
            }
            Card::Cleanup => {
                draw_texture(
                    globals.assets.textures.hex,
                    x + 50.0 - HEX_WIDTH / 2.0,
                    y + 70.0 - HEX_HEIGHT / 2.0,
                    WHITE,
                );
            }
        }
    }
}

/// An instruction blueprint on a card
#[derive(Clone)]
pub enum CardInstruction {
    /// Go in *this* direction
    Direct,
    /// Rotate relative by *this* much
    Rotate,
    /// Shunt in *this* direction
    Shunt,
}

impl CardInstruction {
    /// Sample a random CardInstruction
    pub fn sample() -> Self {
        let mut rng = rand::thread_rng();
        let samples = [
            CardInstruction::Direct,
            CardInstruction::Direct,
            CardInstruction::Direct,
            CardInstruction::Shunt,
        ];
        samples[rng.gen_range(0..samples.len())].clone()
    }

    /// Turn this into an Instruction
    pub fn to_instruction(&self, rotation: i32) -> Instruction {
        match self {
            CardInstruction::Direct => Instruction::Direct(Direction::from_int(rotation)),
            CardInstruction::Rotate => Instruction::Rotate(Angle::from_int(rotation)),
            CardInstruction::Shunt => Instruction::Shunt(Direction::from_int(rotation)),
        }
    }
}

/// A card when held by the player.
pub struct SelectedCard {
    /// Original index of the card in hand, or None if it was from the board
    pub original_idx: Option<usize>,
    pub card: Card,
    /// Absolute pixel position of the center
    pub position: (f32, f32),
    /// Rotation amount, also indicates other things...
    /// It's what changes with the mouse wheel.
    pub rotation: i32,
}

impl SelectedCard {
    pub fn draw(&self, cx: f32, cy: f32, globals: &Globals) {
        match &self.card {
            Card::Truck { cargo } => {
                let to_draw = Truck {
                    cargo: cargo.to_owned(),
                    speed: 0.0,
                    facing: Direction::from_int(self.rotation),
                    position: Coordinate::new(0, 0),
                    move_progress: 0.0,
                    out_of_bounds_immunity: false,
                };
                to_draw.draw_absolute(cx, cy, globals);
            }
            Card::Instruction(instr) => {
                let cell = Cell::Instruction(match instr {
                    CardInstruction::Direct => {
                        Instruction::Direct(Direction::from_int(self.rotation))
                    }
                    CardInstruction::Rotate => Instruction::Rotate(Angle::from_int(self.rotation)),
                    CardInstruction::Shunt => {
                        Instruction::Shunt(Direction::from_int(self.rotation))
                    }
                });
                cell.draw_absolute(cx - HEX_WIDTH / 2.0, cy - HEX_HEIGHT / 2.0, globals);
            }
            Card::Cleanup => {
                draw_texture(
                    globals.assets.textures.hex,
                    cx - HEX_WIDTH / 2.0,
                    cy - HEX_HEIGHT / 2.0,
                    WHITE,
                );
            }
        }
    }
}
