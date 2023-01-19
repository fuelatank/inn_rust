use std::cell::Ref;

use serde::{Serialize, Serializer};

use crate::{
    board::Board,
    card::{Achievement, Age, Card, SpecialAchievement},
    game::PlayerId,
    state::ExecutionObs,
    turn::Turn,
};

// lifetime?
type BoardView<'a> = Ref<'a, Board<'a>>;

type CardView<'a> = Vec<&'a Card>;
type AgeView = Vec<Age>;

fn serialize_board<S: Serializer>(board: &BoardView, serializer: S) -> Result<S::Ok, S::Error> {
    board.serialize(serializer)
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", content = "view", rename_all = "snake_case")]
pub enum SingleAchievementView {
    Special(SpecialAchievement),
    Normal(Age),
}

impl From<Age> for SingleAchievementView {
    fn from(v: Age) -> Self {
        Self::Normal(v)
    }
}

impl From<SpecialAchievement> for SingleAchievementView {
    fn from(v: SpecialAchievement) -> Self {
        Self::Special(v)
    }
}

impl<'a> PartialEq<Achievement<'a>> for SingleAchievementView {
    fn eq(&self, other: &Achievement) -> bool {
        match (self, other) {
            (SingleAchievementView::Normal(age), Achievement::Normal(card)) => *age == card.age(),
            (SingleAchievementView::Special(self_sa), Achievement::Special(other_sa)) => {
                self_sa == other_sa
            }
            _ => false,
        }
    }
}

impl<'a> PartialEq<SingleAchievementView> for Achievement<'a> {
    fn eq(&self, other: &SingleAchievementView) -> bool {
        match (self, other) {
            (Achievement::Normal(card), SingleAchievementView::Normal(age)) => card.age() == *age,
            (Achievement::Special(self_sa), SingleAchievementView::Special(other_sa)) => {
                self_sa == other_sa
            }
            _ => false,
        }
    }
}

type AchievementView = Vec<SingleAchievementView>;

pub struct TurnView {
    main_action_index: usize,
}

impl TurnView {
    pub fn is_second_action(&self) -> bool {
        self.main_action_index % 2 == 0
    }
}

#[derive(Debug, Serialize)]
pub struct MainPlayerView<'a> {
    pub hand: CardView<'a>,
    pub score: CardView<'a>,
    #[serde(serialize_with = "serialize_board")]
    pub board: BoardView<'a>,
    pub achievements: AchievementView,
}

#[derive(Debug, Serialize)]
pub struct OtherPlayerView<'a> {
    pub hand: AgeView,
    pub score: AgeView,
    #[serde(serialize_with = "serialize_board")]
    pub board: BoardView<'a>,
    pub achievements: AchievementView,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObsType<'a> {
    Main,
    Executing(ExecutionObs<'a>),
}

#[derive(Debug, Serialize)]
pub struct Observation<'a> {
    /// The player who is making choice.
    pub acting_player: PlayerId,
    pub main_player: MainPlayerView<'a>,
    pub other_players: Vec<OtherPlayerView<'a>>,
    pub main_pile: [usize; 10],
    pub turn: &'a Turn,
    pub obstype: ObsType<'a>,
}

#[derive(Debug, Serialize)]
pub struct EndObservation<'a> {
    // todo: reveal achievement
    pub players_from_current: Vec<MainPlayerView<'a>>,
    pub main_pile: [usize; 10],
    pub turn: &'a Turn,
    pub winners: Vec<PlayerId>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum GameState<'a> {
    Normal(Observation<'a>),
    End(EndObservation<'a>),
}

impl<'a> GameState<'a> {
    pub fn as_normal(self) -> Option<Observation<'a>> {
        if let Self::Normal(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_end(self) -> Option<EndObservation<'a>> {
        if let Self::End(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        card::{Color, Icon},
        state::Choose,
    };

    use super::*;
    use serde_json::{json, to_value};

    #[test]
    fn obstype_serialization() {
        assert_eq!(to_value(&ObsType::Main).unwrap(), json!("main"));
        // MAYFIXED: TODO: test for Executing, actual Card needed
        let card = Card::new_noop("PlaceHolder".to_owned(), 4, Color::Red, [Icon::Empty; 4]);
        let card_value = to_value(&card).unwrap();
        assert_eq!(
            to_value(&ObsType::Executing(ExecutionObs {
                state: Choose::Opponent,
                card: &card,
            }))
            .unwrap(),
            json!({
                "executing": {
                    "state": {
                        "type": "opponent",
                    },
                    "card": card_value,
                }
            })
        );
    }

    #[test]
    fn achievement_serialization() {
        assert_eq!(
            to_value(&SingleAchievementView::Normal(8)).unwrap(),
            json!({
                "type": "normal",
                "view": 8
            })
        );
        assert_eq!(
            to_value(&SingleAchievementView::Special(SpecialAchievement::Wonder)).unwrap(),
            json!({
                "type": "special",
                "view": "Wonder",
            })
        );
    }
}
