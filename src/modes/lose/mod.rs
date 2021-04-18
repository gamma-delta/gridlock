use crate::{
    drawutils::{self, TextAlign},
    GameMode, Globals, Transition,
};

use super::ModePlaying;

const TIME_HERE_TILL_RETURN: u64 = 20;

pub struct ModeLose {
    /// What was your top score?
    highscore: u32,

    frames_elapsed: u64,
}

impl ModeLose {
    pub fn new(highscore: u32) -> Self {
        ModeLose {
            highscore,
            frames_elapsed: 0,
        }
    }

    pub fn update(&mut self, globals: &mut Globals) -> Transition {
        use macroquad::prelude::*;

        let out = if self.frames_elapsed >= TIME_HERE_TILL_RETURN
            && is_mouse_button_pressed(MouseButton::Left)
        {
            Transition::Swap(GameMode::Playing(ModePlaying::new()))
        } else {
            Transition::None
        };
        self.frames_elapsed += 1;
        out
    }

    pub fn draw(&self, globals: &Globals) {
        use macroquad::prelude::*;
        clear_background(Color::from_rgba(250, 252, 255, 255));

        drawutils::text(globals, "GAME OVER", 30, 400.0, 100.0, TextAlign::Center);
        drawutils::text(
            globals,
            format!("Score: ${}", self.highscore).as_str(),
            20,
            400.0,
            200.0,
            TextAlign::Center,
        );
        drawutils::text(
            globals,
            "Click anywhere to play again",
            20,
            400.0,
            400.0,
            TextAlign::Center,
        );
    }
}
