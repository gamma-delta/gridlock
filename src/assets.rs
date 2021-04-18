#![allow(clippy::eval_order_dependence)]

use std::path::PathBuf;

use macroquad::prelude::*;
use once_cell::sync::Lazy;

pub struct Assets {
    pub textures: Textures,

    pub font: Font,
}

impl Assets {
    pub async fn init() -> Self {
        Self {
            textures: Textures::init().await,

            font: load_ttf_font(
                ASSETS_ROOT
                    .join("OpenSans-SemiBold.ttf")
                    .to_string_lossy()
                    .as_ref(),
            )
            .await,
        }
    }
}

pub struct Textures {
    pub truck: Texture2D,
    pub treads: Texture2D,

    pub hex: Texture2D,
    pub factory: Texture2D,
    pub market: Texture2D,
    pub wreckage: Texture2D,

    pub instr_direct: Texture2D,
    pub instr_shunt: Texture2D,
    pub instr_rotate0: Texture2D,
    pub instr_rotate1: Texture2D,
    pub instr_rotate2: Texture2D,
    pub instr_rotate3: Texture2D,

    pub card: Texture2D,
    pub delete_card: Texture2D,

    pub apple: Texture2D,
    pub orange: Texture2D,
    pub lemon: Texture2D,
    pub lettuce: Texture2D,
    pub grape: Texture2D,
    pub potato: Texture2D,
    pub any_item: Texture2D,
}

impl Textures {
    pub async fn init() -> Self {
        Self {
            truck: texture("truck").await,
            treads: texture("treads").await,

            hex: texture("hex").await,
            factory: texture("factory").await,
            market: texture("market").await,
            wreckage: texture("wreckage").await,

            instr_direct: texture("instructions/direct").await,
            instr_shunt: texture("instructions/shunt").await,
            instr_rotate0: texture("instructions/rotate_0").await,
            instr_rotate1: texture("instructions/rotate_1").await,
            instr_rotate2: texture("instructions/rotate_2").await,
            instr_rotate3: texture("instructions/rotate_3").await,

            card: texture("card").await,
            delete_card: texture("card_delete").await,

            apple: texture("items/apple").await,
            orange: texture("items/orange").await,
            lemon: texture("items/lemon").await,
            lettuce: texture("items/lettuce").await,
            grape: texture("items/grape").await,
            potato: texture("items/potato").await,
            any_item: texture("items/any").await,
        }
    }
}

async fn texture(path: &str) -> Texture2D {
    let with_extension = path.to_owned() + ".png";
    load_texture(
        ASSETS_ROOT
            .join("textures")
            .join(with_extension)
            .to_string_lossy()
            .as_ref(),
    )
    .await
    .unwrap()
}

static ASSETS_ROOT: Lazy<PathBuf> = Lazy::new(|| {
    if cfg!(debug_assertions) {
        if cfg!(target_arch = "wasm32") {
            PathBuf::from("../assets")
        } else {
            PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets"))
        }
    } else {
        todo!("assets path for release hasn't been finalized yet ;-;")
    }
});
