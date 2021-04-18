#![feature(drain_filter)]

mod assets;
pub mod drawutils;
mod modes;
use assets::Assets;
use modes::{ModeLose, ModePlaying};

use macroquad::prelude::*;

fn conf() -> Conf {
    Conf {
        window_title: String::from("Gridlock"),
        window_width: 900,
        window_height: 1000,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut modes = vec![GameMode::Playing(ModePlaying::new())];
    let mut globals = Globals::new().await;

    loop {
        let trans = match modes.last_mut().unwrap() {
            GameMode::Playing(mode) => mode.update(&mut globals),
            GameMode::Lose(mode) => mode.update(&mut globals),
        };
        match trans {
            Transition::Push(mode) => modes.push(mode),
            Transition::Pop => {
                if modes.len() >= 2 {
                    modes.pop();
                }
            }
            Transition::Swap(mode) => *modes.last_mut().unwrap() = mode,
            Transition::None => {}
        }

        match modes.last().unwrap() {
            GameMode::Playing(mode) => mode.draw(&globals),
            GameMode::Lose(mode) => mode.draw(&globals),
        }

        next_frame().await
    }
}

pub enum GameMode {
    Playing(ModePlaying),
    Lose(ModeLose),
}

pub struct Globals {
    assets: Assets,
}

impl Globals {
    pub async fn new() -> Self {
        Self {
            assets: Assets::init().await,
        }
    }
}

pub enum Transition {
    /// Do nothing
    None,
    /// Push this state on top
    Push(GameMode),
    /// Remove the current state and replace it with this
    Swap(GameMode),
    /// Pop the current state off
    Pop,
}
