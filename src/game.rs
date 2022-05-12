use crate::board::Board;
use crate::card::Achievement;
use crate::card::{Card, Dogma};
use crate::card_pile::MainCardPile;
use crate::containers::{Addable, CardSet, Removeable};
use crate::enums::{Color, Splay};
use crate::flow::{Action, State};
use generator::{Gn, LocalGenerator};
use std::cell::RefCell;

pub type BoxCardSet<'a> = Box<dyn CardSet<'a, Card>>;
pub type BoxAchievementSet<'a> = Box<dyn CardSet<'a, Achievement>>;

pub struct Player<'a> {
    id: usize,
    main_pile: &'a RefCell<MainCardPile<'a>>,
    main_board: RefCell<Board<'a>>,
    pub hand: RefCell<BoxCardSet<'a>>,
    pub score_pile: RefCell<BoxCardSet<'a>>,
    achievements: RefCell<BoxAchievementSet<'a>>,
}

impl<'a> Player<'a> {
    fn new(
        id: usize,
        main_pile: &'a RefCell<MainCardPile<'a>>,
        hand: BoxCardSet<'a>,
        score_pile: BoxCardSet<'a>,
        achievements: BoxAchievementSet<'a>,
    ) -> Player<'a> {
        Player {
            id,
            main_pile: main_pile,
            main_board: RefCell::new(Board::new()),
            hand: RefCell::new(hand),
            score_pile: RefCell::new(score_pile),
            achievements: RefCell::new(achievements),
        }
    }

    pub fn age(&self) -> u8 {
        self.main_board.borrow().highest_age()
    }

    pub fn board(&self) -> &RefCell<Board<'a>> {
        &self.main_board
    }

    pub fn draw(&self, age: u8) -> Option<&'a Card> {
        transfer(self.main_pile, &self.hand, &age)
    }

    pub fn draw_and_meld(&self, age: u8) -> Option<&'a Card> {
        transfer(self.main_pile, &self.main_board, &age)
    }

    pub fn draw_and_score(&self, age: u8) -> Option<&'a Card> {
        transfer(self.main_pile, &self.score_pile, &age)
    }

    pub fn score(&self, card: &'a Card) -> Option<&'a Card> {
        transfer(&self.hand, &self.score_pile, card)
    }

    pub fn tuck(&self, card: &'a Card) -> Option<&'a Card> {
        transfer(&self.hand, &self.main_board, card)
    }

    pub fn splay(&self, color: Color, direction: Splay) {
        self.main_board
            .borrow_mut()
            .get_stack_mut(color)
            .splay(direction);
    }

    pub fn is_splayed(&self, color: Color, direction: Splay) -> bool {
        self.main_board.borrow().is_splayed(color, direction)
    }

    pub fn r#return(&self, card: &'a Card) -> Option<&'a Card> {
        transfer(&self.hand, self.main_pile, card)
    }

    pub fn execute<'g>(
        &'g self,
        card: &'a Card,
        game: &'g Game<'a>,
    ) -> LocalGenerator<'g, Action<'a, 'g>, State<'a, 'g>> {
        Gn::new_scoped_local(move |mut s| {
            let main_icon = card.main_icon();
            for dogma in card.dogmas() {
                match dogma {
                    Dogma::Share(flow) => {
                        // should filter out ineligible players
                        for player in game.players(self.id) {
                            let mut gen = flow(player, game);
                            // s.yield_from(gen);
                            let mut state = gen.resume();
                            while let Some(st) = state {
                                let a = s.yield_(st).expect("Generator got None");
                                gen.set_para(a);
                                state = gen.resume();
                            }
                        }
                    }
                    Dogma::Demand(_) => {}
                }
            }
            generator::done!()
        })
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
        let id = self.players.len();
        self.players.push(Player::new(
            id,
            &self.main_card_pile,
            hand,
            score_pile,
            achievements,
        ))
    }

    pub fn player(&self, index: usize) -> &Player<'a> {
        &self.players[index]
    }

    pub fn players(&self, main_player_id: usize) -> impl Iterator<Item = &Player<'a>> {
        (0..self.players.len())
            .map(move |i| &self.players[(i + main_player_id) % self.players.len()])
    }

    /* pub fn pile(&self) -> &MainCardPile<'a> {
        &self.main_card_pile
    } */
}

pub fn transfer<'a, T, P, R, S>(from: &RefCell<R>, to: &RefCell<S>, param: &P) -> Option<&'a T>
where
    R: Removeable<'a, T, P>,
    S: Addable<'a, T>,
{
    let c = from.borrow_mut().remove(param);
    if let Some(card) = c {
        to.borrow_mut().add(card);
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
