use std::cell::Ref;

use serde::{Serialize, Serializer};

use crate::{
    board::Board,
    card::{Card, SpecialAchievement},
    game::Turn,
    state::ExecutionObs,
};

// lifetime?
type BoardView<'a> = Ref<'a, Board<'a>>;

type CardView<'a> = Vec<&'a Card>;
type AgeView = Vec<u8>;

fn serialize_board<S: Serializer>(board: &BoardView, serializer: S) -> Result<S::Ok, S::Error> {
    board.serialize(serializer)
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "view", rename_all = "snake_case")]
pub enum SingleAchievementView<'a> {
    Special(&'a SpecialAchievement),
    Normal(u8),
}

type AchievementView<'a> = Vec<SingleAchievementView<'a>>;

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
    pub achievements: AchievementView<'a>,
}

#[derive(Debug, Serialize)]
pub struct OtherPlayerView<'a> {
    pub hand: AgeView,
    pub score: AgeView,
    #[serde(serialize_with = "serialize_board")]
    pub board: BoardView<'a>,
    pub achievements: AchievementView<'a>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObsType<'a> {
    Main,
    Executing(ExecutionObs<'a>),
}

#[derive(Debug, Serialize)]
pub struct Observation<'a> {
    pub main_player: MainPlayerView<'a>,
    pub other_players: Vec<OtherPlayerView<'a>>,
    pub main_pile: [usize; 10],
    pub turn: &'a Turn,
    pub obstype: ObsType<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{to_value, json};

    #[test]
    fn obstype_serialization() {
        assert_eq!(to_value(&ObsType::Main).unwrap(), json!("main"));
        // todo: test for Executing, actual Card needed
    }

    #[test]
    fn achievement_serialization() {
        assert_eq!(to_value(&SingleAchievementView::Normal(8)).unwrap(), json!({
            "type": "normal",
            "view": 8
        }));
        assert_eq!(to_value(&SingleAchievementView::Special(&SpecialAchievement::Wonder)).unwrap(), json!({
            "type": "special",
            "view": "Wonder",
        }));
    }
}
