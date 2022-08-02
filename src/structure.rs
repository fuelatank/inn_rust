use crate::{
    card::Card,
    enums::Color,
    error::{InnResult, InnovationError},
    player::Player,
};

#[derive(Copy, Clone)]
pub enum PlayerPlace {
    Hand,
    Score,
    Board,
}

#[derive(Copy, Clone)]
pub enum Place {
    MainCardPile,
    Player(usize, PlayerPlace),
}

impl Place {
    pub fn hand<'c, 'g>(player: &'g Player<'c>) -> Place {
        Place::Player(player.id(), PlayerPlace::Hand)
    }

    pub fn score<'c, 'g>(player: &'g Player<'c>) -> Place {
        Place::Player(player.id(), PlayerPlace::Score)
    }

    pub fn board<'c, 'g>(player: &'g Player<'c>) -> Place {
        Place::Player(player.id(), PlayerPlace::Board)
    }
}

pub enum RemoveParam<'c> {
    Age(u8),
    Card(&'c Card),
    Top(bool),
    ColoredTop(Color, bool),
    Index(usize),
    ColoredIndex(Color, usize),
    NoParam,
}

pub enum AddParam {
    Top(bool),
    Index(usize),
    NoParam,
}

impl<'c> RemoveParam<'c> {
    pub fn age(self) -> InnResult<u8> {
        if let RemoveParam::Age(age) = self {
            Ok(age)
        } else {
            Err(InnovationError::ParamUnwrapError)
        }
    }
}
