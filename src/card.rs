
use crate::enums::{Color, Icon};

pub struct Dogma {
    demand: bool,
    flow: Flow
}

pub struct Card {
    name: String,
    age: u8,
    color: Color,
    icons: [Icon; 4],
    dogmas: Vec<Dogma>,
    doc: String
}

impl Card {
    pub fn age(&self) -> u8 {
        self.age
    }
    pub fn color(&self) -> Color {
        self.color
    }
}

pub enum SpecialAchievement {
    Universe,
    Wonder,
    World,
    // TODO: more
}

pub enum Achievement {
    Normal(Card),
    Special(SpecialAchievement)
}