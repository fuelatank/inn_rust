use serde::Deserialize;

use crate::game::Players;
use crate::player::Player;
use crate::{card::Card, card_attrs::Age};

#[derive(Clone)]
pub enum RefStepAction<'c> {
    Draw,
    Meld(&'c Card),
    Achieve(Age),
    Execute(&'c Card),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoRefStepAction {
    Draw,
    Meld(String),
    Achieve(Age),
    Execute(String),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoRefChoice {
    Card(Vec<String>),
    Opponent(usize),
    Yn(bool),
}

pub enum RefChoice<'c, 'g> {
    Card(Vec<&'c Card>),
    Opponent(&'g Player<'c>),
    Yn(bool),
    NoValidAction,
}

impl<'c, 'g> RefChoice<'c, 'g> {
    pub fn card(self) -> Option<&'c Card> {
        match self {
            RefChoice::Card(cards) => {
                if cards.len() == 1 {
                    Some(cards[0])
                } else {
                    panic!("Error when unwrapping Action to one card")
                }
            }
            RefChoice::NoValidAction => None,
            _ => {
                panic!("Error when unwrapping Action to one card")
            }
        }
    }

    pub fn cards(self) -> Option<Vec<&'c Card>> {
        match self {
            RefChoice::Card(cards) => Some(cards),
            RefChoice::NoValidAction => None,
            _ => {
                panic!("Error when unwrapping Action to cards")
            }
        }
    }

    pub fn player(self) -> Option<&'g Player<'c>> {
        match self {
            RefChoice::Opponent(player) => Some(player),
            RefChoice::NoValidAction => None,
            _ => panic!("Error when unwrapping Action to player"),
        }
    }

    pub fn yn(self) -> Option<bool> {
        match self {
            RefChoice::Yn(yn) => Some(yn),
            RefChoice::NoValidAction => None,
            _ => panic!("Error when unwrapping Action to yn"),
        }
    }
}

pub enum RefAction<'c, 'g> {
    Step(RefStepAction<'c>),
    Executing(RefChoice<'c, 'g>),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;
    use Action::*;
    use NoRefChoice::*;
    use NoRefStepAction::*;

    #[test]
    fn action_deserialization() {
        matches!(from_str("draw"), Ok(Step(Draw)));
        matches!(from_str("{ \"meld\": \"Agriculture\" }"), Ok(Step(Meld(x))) if x == "Agriculture");
        matches!(from_str("{ \"achieve\": 8 }"), Ok(Step(Achieve(8))));
        matches!(from_str("{ \"execute\": \"Tools\" }"), Ok(Step(Execute(x))) if x == "Tools");
        matches!(from_str("{ \"card\": [\"Pottery\"] }"), Ok(Executing(Card(x))) if x == vec!["Pottery"]);
        matches!(from_str("{ \"opponent\": 1 }"), Ok(Executing(Opponent(1))));
        matches!(from_str("{ \"yn\": true }"), Ok(Executing(Yn(true))));
    }
}
