use crate::modes::playing::HAND_SIZE;
use macroquad::prelude::*;

use crate::Globals;

pub const BOARD_ORIGIN_X: f32 = 450.0;
pub const BOARD_ORIGIN_Y: f32 = 400.0;

pub const HEX_WIDTH: f32 = 64.0;
pub const HEX_HEIGHT: f32 = HEX_RADIUS * 2.0;
pub const HEX_RADIUS: f32 = HEX_WIDTH / 1.7320508; // sqrt 3

pub const EMOTE_SIZE: f32 = 618.0;

pub const CARD_WIDTH: f32 = 100.0;
pub const CARD_HEIGHT: f32 = CARD_WIDTH * 7.0 / 5.0;
pub const CARD_TOP_POS: f32 = 1000.0 - CARD_HEIGHT - 20.0;
pub const CARD_PADDING: f32 = 10.0;

pub const HUD_LEFT_POS: f32 = (CARD_WIDTH + CARD_PADDING) * (HAND_SIZE + 1) as f32;

pub enum TextAlign {
    Left,
    Center,
    Right,
}

// thanks alwinfy my greatest palwinfy
pub fn center_text(globals: &Globals, text: &str, size: u16, cx: f32, cy: f32) {
    let center_y = cy - size as f32 * (text.lines().count() as f32 - 5. / 3.) * 0.5;
    self::text(globals, text, size, cx, center_y, TextAlign::Center);
}
pub fn center_text_color(globals: &Globals, text: &str, size: u16, cx: f32, cy: f32, color: Color) {
    let center_y = cy - size as f32 * (text.lines().count() as f32 - 5. / 3.) * 0.5;
    self::text_color(globals, text, size, cx, center_y, TextAlign::Center, color);
}

/// Draw the text with the given size at the given position.
pub fn text(globals: &Globals, text: &str, size: u16, cx: f32, cy: f32, align: TextAlign) {
    self::text_color(globals, text, size, cx, cy, align, BLACK);
}

pub fn text_color(
    globals: &Globals,
    text: &str,
    size: u16,
    cx: f32,
    cy: f32,
    align: TextAlign,
    color: Color,
) {
    let params = TextParams {
        font_size: size,
        font: globals.assets.font,
        color,
        ..Default::default()
    };
    for (idx, line) in text.lines().enumerate() {
        let offset = match align {
            TextAlign::Left => 0.0,
            TextAlign::Center => 0.5,
            TextAlign::Right => 1.0,
        };
        let width = measure_text(line, Some(globals.assets.font), size, 1.).width;
        draw_text_ex(
            line,
            cx - offset * width,
            cy + size as f32 * idx as f32,
            params,
        );
    }
}
