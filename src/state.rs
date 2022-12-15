use serde::Serialize;

use crate::card::Card;
use crate::flow::FlowState;
use crate::player::Player;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Choose<'a> {
    Card {
        min_num: u8,
        max_num: Option<u8>,
        from: Vec<&'a Card>,
    },
    Opponent,
    Yn,
}

pub struct ExecutionState<'c, 'g> {
    actor: &'g Player<'c>,
    state: Choose<'c>,
    card: Option<&'c Card>, // option because a dogma doesn't know which card it belongs to
}

impl<'c, 'g> ExecutionState<'c, 'g> {
    pub fn new(actor: &'g Player<'c>, state: Choose<'c>) -> ExecutionState<'c, 'g> {
        ExecutionState {
            actor,
            state,
            card: None,
        }
    }

    pub fn or(self, card: &'c Card) -> ExecutionState<'c, 'g> {
        ExecutionState {
            card: self.card.or(Some(card)),
            ..self
        }
    }

    pub fn to_obs(self) -> (&'g Player<'c>, ExecutionObs<'c>) {
        (
            self.actor,
            ExecutionObs {
                state: self.state,
                card: self.card.unwrap(),
            },
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ExecutionObs<'c> {
    pub state: Choose<'c>,
    pub card: &'c Card,
}

pub enum State<'c, 'g> {
    Main,
    Executing(FlowState<'c, 'g>),
}

impl<'c, 'g> Default for State<'c, 'g> {
    fn default() -> Self {
        State::Main
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{to_value, json};

    #[test]
    fn choose_serialization() {
        // println!("{}", to_string(&Choose::Card { min_num: 3, max_num: None, from: vec![] }).unwrap());
        assert_eq!(to_value(Choose::Card { min_num: 3, max_num: None, from: vec![] }).unwrap(), json!({
            "type": "card",
            "min_num": 3,
            "max_num": null,
            "from": [],
        }));
        assert_eq!(to_value(Choose::Card { min_num: 3, max_num: Some(4), from: vec![] }).unwrap(), json!({
            "type": "card",
            "min_num": 3,
            "max_num": 4,
            "from": [],
        }));
    }
}