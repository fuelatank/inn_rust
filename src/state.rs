use std::cmp::min;

use serde::Serialize;

use crate::flow::{FlowState, GenResume};
use crate::player::Player;
use crate::{action::RefChoice, card::Card, game::Players};

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Choose<'c> {
    // maybe usize?
    Card {
        min_num: u8,
        max_num: Option<u8>,
        from: Vec<&'c Card>,
    },
    Opponent,
    Yn,
}

pub enum ActionCheckResult<'c, 'g> {
    Zero,
    One(GenResume<'c, 'g>),
    Many,
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

    pub fn check_valid_actions(&self, game: &'g Players<'c>) -> ActionCheckResult<'c, 'g> {
        match self.state {
            Choose::Card {
                min_num,
                max_num,
                ref from,
            } => {
                let len = from.len();
                let real_max_num = if let Some(max_num) = max_num {
                    min(len, max_num.into())
                } else {
                    len
                };
                let min_num: usize = min_num.into();

                // min_num <= n <= real_max_num
                // iff
                // there's a valid choice with length n

                if real_max_num < min_num {
                    // no valid action
                    return ActionCheckResult::Zero;
                }

                if real_max_num == min_num {
                    // there're (len choose min_num) possibilities
                    // which equals 1 iff min_num == 0 or min_num == len
                    if min_num == 0 {
                        return ActionCheckResult::One(RefChoice::Card(Vec::new()));
                    }
                    if min_num == len {
                        return ActionCheckResult::One(RefChoice::Card(from.clone()));
                    }
                }
            }
            Choose::Opponent => {
                if game.num_players() == 2 {
                    return ActionCheckResult::One(RefChoice::Opponent(
                        game.players_from(self.actor.id() + 1).next().unwrap(),
                    ));
                }
            }
            Choose::Yn => {}
        }
        ActionCheckResult::Many
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
    use serde_json::{json, to_value};

    #[test]
    fn choose_serialization() {
        // println!("{}", to_string(&Choose::Card { min_num: 3, max_num: None, from: vec![] }).unwrap());
        assert_eq!(
            to_value(Choose::Card {
                min_num: 3,
                max_num: None,
                from: vec![]
            })
            .unwrap(),
            json!({
                "type": "card",
                "min_num": 3,
                "max_num": null,
                "from": [],
            })
        );
        assert_eq!(
            to_value(Choose::Card {
                min_num: 3,
                max_num: Some(4),
                from: vec![]
            })
            .unwrap(),
            json!({
                "type": "card",
                "min_num": 3,
                "max_num": 4,
                "from": [],
            })
        );
    }
}
