
pub enum Color {
    Blue,
    Red,
    Green,
    Yellow,
    Purple
}

impl Color {
    pub fn as_usize(&self) -> usize {
        match self {
            Color::Blue => 0,
            Color::Red => 1,
            Color::Green => 2,
            Color::Yellow => 3,
            Color::Purple => 4
        }
    }
}

pub enum Icon {
    Castle,
    Factory,
    Clock,
    Crown,
    Lightblub,
    Leaf
}

pub enum Splay {
    NoSplay,
    Left,
    Right,
    Up
}