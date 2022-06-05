use std::cell::Ref;

use crate::{
    board::Board,
    card::{Card, SpecialAchievement},
    state::{Choose, ExecutionObs}, game::Turn,
};

// lifetime?
type BoardView<'a> = Ref<'a, Board<'a>>;

type CardView<'a> = Vec<&'a Card>;
type AgeView = Vec<u8>;

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

pub struct MainPlayerView<'a> {
    pub hand: CardView<'a>,
    pub score: CardView<'a>,
    pub board: BoardView<'a>,
    pub achievements: AchievementView<'a>,
}

pub struct OtherPlayerView<'a> {
    pub hand: AgeView,
    pub score: AgeView,
    pub board: BoardView<'a>,
    pub achievements: AchievementView<'a>,
}

pub enum ObsType<'a> {
    Main,
    Executing(ExecutionObs<'a>),
}

pub struct Observation<'a> {
    pub main_player: MainPlayerView<'a>,
    pub other_players: Vec<OtherPlayerView<'a>>,
    pub main_pile: [usize; 10],
    pub turn: &'a Turn,
    pub obstype: ObsType<'a>,
}
