use std::cell::Ref;

use crate::{
    board::Board,
    card::{Card, SpecialAchievement},
    state::Choose,
};

// lifetime?
type BoardView<'a> = Ref<'a, Board<'a>>;

pub struct CardView<'a>(Vec<&'a Card>);
pub struct AgeView(Vec<u8>);

pub enum SingleAchievementView {
    Special(SpecialAchievement),
    Normal(u8),
}

pub struct AchievementView(Vec<SingleAchievementView>);

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
    pub achievements: AchievementView,
}

pub struct OtherPlayerView<'a> {
    pub hand: AgeView,
    pub score: AgeView,
    pub board: BoardView<'a>,
    pub achievements: AchievementView,
}

pub enum ObsType<'a> {
    Main,
    Executing(&'a Card, Choose<'a>),
}

pub struct Observation<'a> {
    pub main_player: MainPlayerView<'a>,
    pub other_players: Vec<OtherPlayerView<'a>>,
    pub main_pile: [usize; 10],
    pub turn: TurnView,
    pub obstype: ObsType<'a>,
}
