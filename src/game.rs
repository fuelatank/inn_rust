use crate::card_pile::MainCardPile;
use crate::containers::{BoxAchievementSet, BoxCardSet};
use crate::player::Player;
use generator::LocalGenerator;
use std::cell::RefCell;

pub struct Game<'a> {
    main_card_pile: RefCell<MainCardPile<'a>>,
    players: Vec<Player<'a>>,
}

impl<'a> Game<'a> {
    pub fn new() -> Game<'a> {
        Game {
            main_card_pile: RefCell::new(MainCardPile::new()),
            players: vec![],
        }
    }

    pub fn add_player(
        &'a mut self,
        hand: BoxCardSet<'a>,
        score_pile: BoxCardSet<'a>,
        achievements: BoxAchievementSet<'a>,
    ) {
        let id = self.players.len();
        self.players.push(Player::new(
            id,
            &self.main_card_pile,
            hand,
            score_pile,
            achievements,
        ))
    }

    pub fn players(&self, main_player_id: usize) -> impl Iterator<Item = &Player<'a>> {
        (0..self.players.len())
            .map(move |i| &self.players[(i + main_player_id) % self.players.len()])
    }
}

mod tests {
    use super::*;
    use crate::containers::VecSet;

    #[test]
    fn create_game_player() {
        let mut game = Game::new();
        game.add_player(
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
        );
    }
}
