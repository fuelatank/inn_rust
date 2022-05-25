use std::rc::Rc;
use crate::board::Board;
use crate::card::{Card, Dogma};
use crate::card_pile::MainCardPile;
use crate::containers::{transfer, BoxAchievementSet, BoxCardSet};
use crate::enums::{Color, Splay};
use crate::flow::FlowState;
use crate::game::{RcCell, InnerGame};
use generator::{Gn, LocalGenerator};
use std::cell::RefCell;

pub struct Player<'c> {
    id: usize,
    main_pile: RcCell<MainCardPile<'c>>,
    main_board: RefCell<Board<'c>>,
    pub hand: RefCell<BoxCardSet<'c>>,
    pub score_pile: RefCell<BoxCardSet<'c>>,
    achievements: RefCell<BoxAchievementSet<'c>>,
}

impl<'c> Player<'c> {
    pub fn new(
        id: usize,
        main_pile: RcCell<MainCardPile<'c>>,
        hand: BoxCardSet<'c>,
        score_pile: BoxCardSet<'c>,
        achievements: BoxAchievementSet<'c>,
    ) -> Player<'c> {
        Player {
            id,
            main_pile,
            main_board: RefCell::new(Board::new()),
            hand: RefCell::new(hand),
            score_pile: RefCell::new(score_pile),
            achievements: RefCell::new(achievements),
        }
    }

    pub fn age(&self) -> u8 {
        self.main_board.borrow().highest_age()
    }

    pub fn board(&self) -> &RefCell<Board<'c>> {
        &self.main_board
    }

    pub fn draw(&self, age: u8) -> Option<&'c Card> {
        transfer(Rc::clone(&self.main_pile), &self.hand, &age)
    }

    pub fn draw_and_meld(&self, age: u8) -> Option<&'c Card> {
        transfer(Rc::clone(&self.main_pile), &self.main_board, &age)
    }

    pub fn draw_and_score(&self, age: u8) -> Option<&'c Card> {
        transfer(Rc::clone(&self.main_pile), &self.score_pile, &age)
    }

    pub fn meld(&self, card: &'c Card) -> Option<&'c Card> {
        transfer(&self.hand, &self.main_board, card)
    }

    pub fn score(&self, card: &'c Card) -> Option<&'c Card> {
        transfer(&self.hand, &self.score_pile, card)
    }

    pub fn tuck(&self, card: &'c Card) -> Option<&'c Card> {
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

    pub fn r#return(&self, card: &'c Card) -> Option<&'c Card> {
        transfer(&self.hand, Rc::clone(&self.main_pile), card)
    }

    pub fn execute<'g>(
        &'g self,
        card: &'c Card,
        game: &'g InnerGame<'c>,
    ) -> FlowState<'c, 'g> {
        Gn::new_scoped_local(move |mut s| {
            let _main_icon = card.main_icon();
            for dogma in card.dogmas() {
                match dogma {
                    Dogma::Share(flow) => {
                        // should filter out ineligible players
                        for player in game.players_from(self.id) {
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
                    Dogma::Demand(flow) => {
                        // should filter out ineligible players
                        for player in game.players_from(self.id) {
                            let mut gen = flow(self, player, game);
                            // s.yield_from(gen);
                            let mut state = gen.resume();
                            while let Some(st) = state {
                                let a = s.yield_(st).expect("Generator got None");
                                gen.set_para(a);
                                state = gen.resume();
                            }
                        }
                    }
                }
            }
            generator::done!()
        })
    }
}
