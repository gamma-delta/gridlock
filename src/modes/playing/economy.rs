use macroquad::prelude::Texture2D;
use rand::Rng;

use crate::Globals;

/// Items that can be bought and sold
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    RedApple,
    Orange,
    YellowLemon,
    GreenLettuce,
    PurpleGrape,
    BrownTater,
}

impl Item {
    /// Get this item's texture
    pub fn texture(&self, globals: &Globals) -> Texture2D {
        let t = &globals.assets.textures;
        match self {
            Item::RedApple => t.apple,
            Item::Orange => t.orange,
            Item::YellowLemon => t.lemon,
            Item::GreenLettuce => t.lettuce,
            Item::PurpleGrape => t.grape,
            Item::BrownTater => t.potato,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Item::RedApple => "Apple",
            Item::Orange => "Orange",
            Item::YellowLemon => "Lemon",
            Item::GreenLettuce => "Lettuce",
            Item::PurpleGrape => "Grape",
            Item::BrownTater => "Potato",
        }
    }

    /// Sample a random Item
    pub fn sample() -> Self {
        let mut rng = rand::thread_rng();
        [
            Item::RedApple,
            Item::Orange,
            Item::YellowLemon,
            Item::GreenLettuce,
            Item::PurpleGrape,
            Item::BrownTater,
        ][rng.gen_range(0..6)]
        .clone()
    }
}

/// Indicates what a Market is interested in.
#[derive(Debug)]
pub enum ItemFilter {
    /// Any item is OK
    Any,
    /// A specifically wanted item
    Specific(Item),
}

impl ItemFilter {
    /// Check if this matches the given item
    pub fn matches(&self, checkee: &Item) -> bool {
        match self {
            ItemFilter::Any => true,
            ItemFilter::Specific(other) => checkee == other,
        }
    }

    pub fn texture(&self, globals: &Globals) -> Texture2D {
        match self {
            ItemFilter::Any => globals.assets.textures.any_item,
            ItemFilter::Specific(item) => item.texture(globals),
        }
    }

    pub fn sample() -> Self {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.2) {
            ItemFilter::Any
        } else {
            ItemFilter::Specific(Item::sample())
        }
    }
}

/// A Factory that produces items.
#[derive(Debug)]
pub struct Factory {
    /// The thing this factory produces
    pub product: Item,
    /// How many it has left
    pub stock: usize,
}

impl Factory {
    pub fn generate(product: Item, frames_elapsed: u64) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            stock: rng.gen_range(1..5),
            product,
        }
    }
}

/// A Market that consumes items.
#[derive(Debug)]
pub struct Market {
    /// What the market wants
    pub request: ItemFilter,
    /// How many more of the item it wants
    pub demand: usize,
    /// Price generator
    pub prices: Pricer,
}

impl Market {
    pub fn generate(request: ItemFilter, frames_elapsed: u64) -> Self {
        let mut rng = rand::thread_rng();
        let is_any = matches!(request, ItemFilter::Any);
        Self {
            request,
            demand: rng.gen_range(2..10),
            prices: Pricer::new(
                (frames_elapsed as f32 / 1000.0).sqrt() / 800.0 * if is_any { 0.5 } else { 1.0 },
                5.0 + rng.gen_range(
                    (frames_elapsed as f32 / 10.0).sqrt()..(frames_elapsed as f32 / 5.0).sqrt(),
                ) * if is_any { 0.2 } else { 1.0 },
            ),
        }
    }
}

/// Price generator
#[derive(Debug)]
pub struct Pricer {
    /// "time" step to sample our equation at
    time: f32,
    /// How much to step by each sample
    dt: f32,
    /// Price multiplier
    multiplier: f32,
    m: f32,
    n: f32,
    o: f32,
}

impl Pricer {
    /// Create a new Pricer with the given multiplier and dt
    pub fn new(dt: f32, multiplier: f32) -> Self {
        let mut rng = rand::thread_rng();
        let m = rng.gen_range(1.0..3.0);
        let n = rng.gen_range(1.0..2.0);
        let o = rng.gen_range(2.0..4.0);

        Self {
            time: 0.0,
            dt,
            multiplier,
            m,
            n,
            o,
        }
    }

    /// Sample the price at the current time.
    pub fn sample(&self) -> u32 {
        // (3 + sin(tm) - cos(tn) - sin(to))/6 for the base price
        let base = 3.0 + (self.time * self.m).sin()
            - (self.time * self.n).cos()
            - (self.time * self.o).sin();
        let out = base / 6.0 * self.multiplier;
        out as u32 + 1
    }
    /// Advance the timer
    pub fn timestep(&mut self) {
        self.time += self.dt;
    }
}
