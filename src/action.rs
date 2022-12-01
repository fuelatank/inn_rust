use crate::card::Card;
use crate::game::Players;
use crate::player::Player;

#[derive(Clone)]
pub enum RefStepAction<'c> {
    Draw,
    Meld(&'c Card),
    Achieve(u8),
    Execute(&'c Card),
}

#[derive(Clone)]
pub enum NoRefStepAction {
    Draw,
    Meld(String),
    Achieve(u8),
    Execute(String),
}

#[derive(Clone)]
pub enum NoRefChoice {
    Card(Vec<String>),
    Opponent(usize),
    Yn(bool),
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

pub enum RefAction<'c, 'g> {
    Step(RefStepAction<'c>),
    Executing(RefChoice<'c, 'g>),
}

#[derive(Clone)]
pub enum Action {
    Step(NoRefStepAction),
    Executing(NoRefChoice),
}

impl Action {
    pub fn to_ref<'c, 'g>(self, game: &'g Players<'c>) -> RefAction<'c, 'g> {
        match self {
            Action::Step(s) => RefAction::Step(match s {
                NoRefStepAction::Draw => RefStepAction::Draw,
                NoRefStepAction::Meld(name) => RefStepAction::Meld(game.find_card(&name)),
                NoRefStepAction::Achieve(a) => RefStepAction::Achieve(a),
                NoRefStepAction::Execute(name) => RefStepAction::Execute(game.find_card(&name)),
            }),
            Action::Executing(e) => RefAction::Executing(match e {
                NoRefChoice::Card(names) => RefChoice::Card(
                    names
                        .into_iter()
                        .map(|name| game.find_card(&name))
                        .collect(),
                ),
                NoRefChoice::Opponent(id) => RefChoice::Opponent(game.player_at(id)),
                NoRefChoice::Yn(yn) => RefChoice::Yn(yn),
            }),
        }
    }
}
