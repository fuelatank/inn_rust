use crate::board::Board;
use crate::card::Achievement;
use crate::card::Card;
use crate::card_pile::MainCardPile;
use crate::containers::{Addable, CardSet, Removeable};
use std::cell::RefCell;
use std::ops::DerefMut;

pub type BoxCardSet<'a> = Box<dyn CardSet<'a, Card>>;
pub type BoxAchievementSet<'a> = Box<dyn CardSet<'a, Achievement>>;

pub struct Player<'a> {
    main_pile: &'a RefCell<MainCardPile<'a>>,
    main_board: Board<'a>,
    pub hand: BoxCardSet<'a>,
    pub score_pile: BoxCardSet<'a>,
    achievements: BoxAchievementSet<'a>,
}

impl<'a> Player<'a> {
    fn new(
        main_pile: &'a RefCell<MainCardPile<'a>>,
        hand: BoxCardSet<'a>,
        score_pile: BoxCardSet<'a>,
        achievements: BoxAchievementSet<'a>,
    ) -> Player<'a> {
        Player {
            main_pile: main_pile,
            main_board: Board::new(),
            hand,
            score_pile,
            achievements,
        }
    }

    pub fn age(&self) -> u8 {
        self.main_board.highest_age()
    }

    pub fn draw(&mut self, age: &'a u8) -> Option<&'a Card> {
        transfer(self.main_pile.borrow_mut(), &mut self.hand, age)
    }

    pub fn draw_and_meld(&mut self, age: &'a u8) -> Option<&'a Card> {
        transfer(self.main_pile.borrow_mut(), &mut self.main_board, age)
    }

    pub fn draw_and_score(&mut self, age: &'a u8) -> Option<&'a Card> {
        transfer(self.main_pile.borrow_mut(), &mut self.score_pile, age)
    }

    pub fn score(&mut self, card: &'a Card) -> Option<&'a Card> {
        transfer(&mut self.hand, &mut self.score_pile, card)
    }

    pub fn r#return(&mut self, card: &'a Card) -> Option<&'a Card> {
        transfer(&mut self.hand, self.main_pile.borrow_mut(), card)
    }
}

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
        self.players.push(Player::new(
            &self.main_card_pile,
            hand,
            score_pile,
            achievements,
        ))
    }

    pub fn player(&self, index: usize) -> &Player<'a> {
        &self.players[index]
    }

    /* pub fn pile(&self) -> &MainCardPile<'a> {
        &self.main_card_pile
    } */
}

pub fn transfer<'a, T, P, R, S, A, B>(mut from: A, mut to: B, param: &'a P) -> Option<&'a T>
where
    R: Removeable<'a, T, P>,
    S: Addable<'a, T>,
    A: DerefMut<Target = R>,
    B: DerefMut<Target = S>,
{
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
        game.add_player(
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
        );
    }
}
