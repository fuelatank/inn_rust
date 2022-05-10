
use crate::card::Card;
use crate::card::Achievement;
use crate::containers::{Addable, Removeable, CardSet};
use crate::board::Board;
use crate::card_pile::MainCardPile;

pub type BoxCardSet<'a> = Box<dyn CardSet<'a, Card>>;
pub type BoxAchievementSet<'a> = Box<dyn CardSet<'a, Achievement>>;

pub struct Player<'a> {
    main_board: Board<'a>,
    pub hand: BoxCardSet<'a>,
    pub score_pile: BoxCardSet<'a>,
    achievements: BoxAchievementSet<'a>
}

impl<'a> Player<'a> {
    fn new(hand: BoxCardSet<'a>, score_pile: BoxCardSet<'a>, achievements: BoxAchievementSet<'a>) -> Player<'a> {
        Player {
            main_board: Board::new(),
            hand,
            score_pile,
            achievements
        }
    }

    pub fn age(&self) -> u8 {
        self.main_board.highest_age()
    }
}

pub struct Game<'a> {
    main_card_pile: MainCardPile<'a>,
    players: Vec<Player<'a>>,
}

impl<'a> Game<'a> {
    pub fn new() -> Game<'a> {
        Game {
            main_card_pile: MainCardPile::new(),
            players: vec![]
        }
    }

    pub fn add_player(&mut self, hand: BoxCardSet<'a>, score_pile: BoxCardSet<'a>, achievements: BoxAchievementSet<'a>) {
        self.players.push(Player::new(hand, score_pile, achievements))
    }

    pub fn player(&self, index: usize) -> &Player<'a> {
        &self.players[index]
    }

    pub fn pile(&self) -> &MainCardPile<'a> {
        &self.main_card_pile
    }

    pub fn draw(&mut self, player: usize, age: &'a u8) -> Option<&'a Card> {
        transfer(&mut self.main_card_pile, &mut self.players[player].hand, age)
    }

    pub fn draw_and_meld(&mut self, player: usize, age: &'a u8) -> Option<&'a Card> {
        transfer(&mut self.main_card_pile, &mut self.players[player].main_board, age)
    }

    pub fn draw_and_score(&mut self, player: usize, age: &'a u8) -> Option<&'a Card> {
        transfer(&mut self.main_card_pile, &mut self.players[player].score_pile, age)
    }

    pub fn score(&mut self, player: usize, card: &'a Card) -> Option<&'a Card> {
        let p = &mut self.players[player];
        transfer(&mut p.hand, &mut p.score_pile, card)
    }

    pub fn r#return(&mut self, player: usize, card: &'a Card) -> Option<&'a Card> {
        transfer(&mut self.players[player].hand, &mut self.main_card_pile, card)
    }
}

fn transfer<'a, T, P>(from: &mut impl Removeable<'a, T, P>, to: &mut impl Addable<'a, T>, param: &'a P) -> Option<&'a T> {
    let c = from.remove(param);
    if let Some(card) = c {
        to.add(card);
    }
    c
}

mod tests {
    use super::*;
    use crate::containers::VecSet;

    #[test]
    fn create_game_player() {
        let mut game = Game::new();
        game.add_player(Box::new(VecSet::default()), Box::new(VecSet::default()), Box::new(VecSet::default()));
    }
}