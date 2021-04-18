use std::f32::consts::TAU;

use hex2d::{Coordinate, Direction, Spacing};

use super::{cells::Cell, economy::Item};
use crate::{
    drawutils::{BOARD_ORIGIN_X, BOARD_ORIGIN_Y, HEX_HEIGHT, HEX_RADIUS, HEX_WIDTH},
    Globals,
};

/// A truck carrying an item around.
#[derive(Debug)]
pub struct Truck {
    /// What it's holding
    pub cargo: Option<Item>,
    /// How far to move per tick.
    pub speed: f32,
    /// Where it's facing
    pub facing: Direction,
    /// The hex this truck is based on.
    pub position: Coordinate,
    /// The fraction this truck is to the next hexagon.
    ///
    ///-  `0.0` = directly on this one
    /// - `1.0` = all the way on the other one
    pub move_progress: f32,
    /// If it's ok to be off the board right now
    pub out_of_bounds_immunity: bool,
}

impl Truck {
    pub fn draw(&self, globals: &Globals) {
        let (cx, cy) = self.get_xy();
        let cx = cx * HEX_WIDTH + BOARD_ORIGIN_X;
        let cy = cy * HEX_WIDTH + BOARD_ORIGIN_Y;
        self.draw_absolute(cx, cy, globals);
    }

    /// Draw this centered at the given pixel coordinates.
    pub fn draw_absolute(&self, cx: f32, cy: f32, globals: &Globals) {
        use macroquad::prelude::*;

        let rotation = self.facing.to_radians_pointy::<f32>() - TAU / 4.0;
        let tx = cx - 32.0;
        let ty = cy - 32.0;
        draw_texture_ex(
            globals.assets.textures.truck,
            tx,
            ty,
            WHITE,
            DrawTextureParams {
                rotation,
                pivot: Some(vec2(cx, cy)),
                ..Default::default()
            },
        );

        if let Some(ref item) = self.cargo {
            draw_texture_ex(
                item.texture(globals),
                cx - 20.0,
                cy - 20.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(40.0, 40.0)),
                    ..Default::default()
                },
            );
        }
    }

    /// Get the hex coordinate this truck's center is over
    pub fn get_hex(&self) -> Coordinate {
        let target = self.position + self.facing;
        if self.move_progress >= 0.5 {
            target
        } else {
            self.position
        }
    }

    /// Get the xy coordinates of the truck.
    ///
    /// - `(0, 0)` is the origin.
    /// - `(1, 0)` is the center of the hex one to the right.
    /// - `(0, 1)` is down one hex-radius from the center.
    pub fn get_xy(&self) -> (f32, f32) {
        // +theta is clockwise in this weird world
        let (dy, dx) = (self.facing.to_radians_pointy::<f32>() - TAU / 4.0).sin_cos();

        let base = self
            .position
            .to_pixel(Spacing::PointyTop(3.0f32.sqrt().recip()));
        let x = base.0 + dx * self.move_progress;
        let y = base.1 + dy * self.move_progress;
        (x, y)
    }

    pub fn from_rot(position: Coordinate, cargo: Option<Item>, rotation: i32) -> Self {
        Truck {
            cargo,
            facing: Direction::from_int(rotation),
            move_progress: 0.0,
            out_of_bounds_immunity: false,
            position,
            speed: 1.0 / 40.0,
        }
    }
}

#[derive(Debug)]
pub struct Treads {
    /// Board-pixel coordinates of the center of this tread
    pub pos: (f32, f32),
    /// Direction the truck was facing
    pub facing: Direction,
    /// Frames left alive
    pub lifetime: u64,
}
