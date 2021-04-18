use super::economy::{Factory, Market};
use crate::{
    drawutils::{self, BOARD_ORIGIN_X, BOARD_ORIGIN_Y, HEX_HEIGHT, HEX_RADIUS, HEX_WIDTH},
    Globals,
};

use hex2d::{Angle, Coordinate, Direction, Spacing};

use std::f32::consts::TAU;

/// A Cell is a spot on a board that trucks can drive on.
#[derive(Debug)]
pub enum Cell {
    Empty,
    /// Oh no, two trucks collided here.
    Wreckage,
    Instruction(Instruction),
    Factory(Factory),
    Market(Market),
}

impl Cell {
    /// Draw this based on the hex position
    pub fn draw(&self, coord: Coordinate, globals: &Globals) {
        // coordinates of the center of the hex
        let (x, y) = coord.to_pixel(Spacing::PointyTop(HEX_RADIUS));
        let x = x + BOARD_ORIGIN_X;
        let y = y + BOARD_ORIGIN_Y;
        // coordinates for the hexagon
        let cx = x - HEX_WIDTH / 2.0;
        let cy = y - HEX_HEIGHT / 2.0;

        self.draw_absolute(cx, cy, globals);
    }

    /// Draw this centered at the given xy coordinates.
    pub fn draw_absolute(&self, cx: f32, cy: f32, globals: &Globals) {
        use macroquad::prelude::*;
        let textures = &globals.assets.textures;
        match self {
            Cell::Empty => {
                draw_texture(textures.hex, cx, cy, WHITE);
            }
            Cell::Factory(factory) => {
                draw_texture(textures.factory, cx, cy, WHITE);
                draw_texture_ex(
                    factory.product.texture(globals),
                    cx + 12.0,
                    cy + 14.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(40.0, 40.0)),
                        ..Default::default()
                    },
                );
                drawutils::center_text(
                    globals,
                    factory.stock.to_string().as_str(),
                    15,
                    cx + 33.0,
                    cy + 58.0,
                );
            }
            Cell::Market(market) => {
                draw_texture(textures.market, cx, cy, WHITE);
                draw_texture_ex(
                    market.request.texture(globals),
                    cx + 12.0,
                    cy + 16.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(40.0, 40.0)),
                        ..Default::default()
                    },
                );
                drawutils::center_text(
                    globals,
                    format!("${}", market.prices.sample()).as_str(),
                    13,
                    cx + 33.0,
                    cy + 15.0,
                );
                drawutils::center_text(
                    globals,
                    market.demand.to_string().as_str(),
                    13,
                    cx + 33.0,
                    cy + 59.0,
                );
            }

            Cell::Wreckage => {
                draw_texture(textures.wreckage, cx, cy, WHITE);
            }

            Cell::Instruction(instruction) => match instruction {
                Instruction::Rotate(rot) => {
                    let textures = &globals.assets.textures;
                    let tex = match rot {
                        Angle::Forward => textures.instr_rotate0,
                        Angle::Left | Angle::Right => textures.instr_rotate1,
                        Angle::LeftBack | Angle::RightBack => textures.instr_rotate2,
                        Angle::Back => textures.instr_rotate3,
                    };
                    let flip = matches!(rot, Angle::Right | Angle::RightBack);
                    draw_texture_ex(
                        tex,
                        cx,
                        cy,
                        WHITE,
                        DrawTextureParams {
                            flip_x: flip,
                            ..Default::default()
                        },
                    );
                }
                Instruction::Direct(dir) => {
                    draw_texture_ex(
                        globals.assets.textures.instr_direct,
                        cx,
                        cy,
                        WHITE,
                        DrawTextureParams {
                            rotation: dir.to_radians_pointy::<f32>() - TAU / 4.0,
                            ..Default::default()
                        },
                    );
                }
                Instruction::Shunt(dir) => {
                    draw_texture_ex(
                        globals.assets.textures.instr_shunt,
                        cx,
                        cy,
                        WHITE,
                        DrawTextureParams {
                            rotation: dir.to_radians_pointy::<f32>() - TAU / 4.0,
                            ..Default::default()
                        },
                    );
                }
            },
        };
    }
}

/// Special instructions you can place on the board.
#[derive(Debug)]
pub enum Instruction {
    /// Rotate the truck relative to its current direction
    Rotate(Angle),
    /// Direct the truck to the given direction
    Direct(Direction),
    /// Move the truck in the direction by one square
    Shunt(Direction),
}
