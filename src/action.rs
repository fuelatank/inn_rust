use crate::card::Card;
use crate::game::InnerGame;
use crate::player::Player;

pub enum StepAction<'c> {
    Draw,
    Meld(&'c Card),
    Achieve(u8),
    Execute(&'c Card),
}

pub enum OuterExecutingAction<'c> {
    Card(Vec<&'c Card>),
    Opponent(usize),
    Yn(bool),
}

impl<'c> OuterExecutingAction<'c> {
    pub fn take_player<'g>(self, game: &'g InnerGame<'c>) -> ExecutingAction<'c, 'g> {
        match self {
            OuterExecutingAction::Card(c) => ExecutingAction::Card(c),
            OuterExecutingAction::Opponent(id) => ExecutingAction::Opponent(game.player_at(id)),
            OuterExecutingAction::Yn(yn) => ExecutingAction::Yn(yn)
        }
    }
}

pub enum ExecutingAction<'c, 'g> {
    Card(Vec<&'c Card>),
    Opponent(&'g Player<'c>),
    Yn(bool),
}

impl<'c, 'g> ExecutingAction<'c, 'g> {
    pub fn card(self) -> Option<&'c Card> {
        if let ExecutingAction::Card(cards) = self {
            if cards.len() == 0 {
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
        if let ExecutingAction::Card(cards) = self {
            cards
        } else {
            panic!("Error when unwrapping Action to cards")
        }
    }

    pub fn player(self) -> &'g Player<'c> {
        if let ExecutingAction::Opponent(player) = self {
            player
        } else {
            panic!("Error when unwrapping Action to player")
        }
    }

    pub fn yn(self) -> bool {
        if let ExecutingAction::Yn(yn) = self {
            yn
        } else {
            panic!("Error when unwrapping Action to yn")
        }
    }
}

pub enum Action<'c> {
    Step(StepAction<'c>),
    Executing(OuterExecutingAction<'c>),
}
