use crate::card::Card;
use crate::game::Players;
use crate::player::Player;

#[derive(Clone)]
pub enum MainAction<'c> {
    Draw,
    Meld(&'c Card),
    Achieve(u8),
    Execute(&'c Card),
}

#[derive(Clone)]
pub enum IdChoice<'c> {
    Card(Vec<&'c Card>),
    Opponent(usize),
    Yn(bool),
}

impl<'c> IdChoice<'c> {
    pub fn to_ref<'g>(self, game: &'g Players<'c>) -> RefChoice<'c, 'g> {
        match self {
            IdChoice::Card(c) => RefChoice::Card(c),
            IdChoice::Opponent(id) => RefChoice::Opponent(game.player_at(id)),
            IdChoice::Yn(yn) => RefChoice::Yn(yn),
        }
    }
}

pub enum RefChoice<'c, 'g> {
    Card(Vec<&'c Card>),
    Opponent(&'g Player<'c>),
    Yn(bool),
}

impl<'c, 'g> RefChoice<'c, 'g> {
    pub fn card(self) -> Option<&'c Card> {
        if let RefChoice::Card(cards) = self {
            if cards.is_empty() {
                None
            } else if cards.len() == 1 {
                Some(cards[0])
            } else {
                panic!("Error when unwrapping Action to one card")
            }
        } else {
            panic!("Error when unwrapping Action to one card")
        }
    }

    pub fn cards(self) -> Vec<&'c Card> {
        if let RefChoice::Card(cards) = self {
            cards
        } else {
            panic!("Error when unwrapping Action to cards")
        }
    }

    pub fn player(self) -> &'g Player<'c> {
        if let RefChoice::Opponent(player) = self {
            player
        } else {
            panic!("Error when unwrapping Action to player")
        }
    }

    pub fn yn(self) -> bool {
        if let RefChoice::Yn(yn) = self {
            yn
        } else {
            panic!("Error when unwrapping Action to yn")
        }
    }
}

#[derive(Clone)]
pub enum Action<'c> {
    Step(MainAction<'c>),
    Executing(IdChoice<'c>),
}
