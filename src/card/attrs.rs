use serde::Serialize;
use strum_macros::EnumIter;

pub type Age = u8;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Serialize)]
pub enum Color {
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
}

impl Color {
    pub fn as_usize(&self) -> usize {
        match self {
            Color::Blue => 0,
            Color::Red => 1,
            Color::Green => 2,
            Color::Yellow => 3,
            Color::Purple => 4,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize)]
pub enum Icon {
    Castle,
    Factory,
    Clock,
    Crown,
    Lightblub,
    Leaf,
    Empty,
}

#[derive(Debug, PartialEq, Default, Clone, Copy, EnumIter, Serialize)]
pub enum Splay {
    #[default]
    NoSplay,
    Left,
    Right,
    Up,
}

impl Splay {
    pub fn mask(&self) -> [bool; 4] {
        // true means shown
        match self {
            Splay::NoSplay => [false, false, false, false],
            Splay::Left => [false, false, false, true],
            Splay::Right => [true, true, false, false],
            Splay::Up => [false, true, true, true],
        }
    }
}
