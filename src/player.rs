use crate::board::Board;
use crate::card::{Card, Dogma};
use crate::card_pile::MainCardPile;
use crate::containers::{transfer, BoxAchievementSet, BoxCardSet};
use crate::enums::{Color, Splay};
use crate::flow::{Action, State};
use crate::game::Game;
use generator::{Gn, LocalGenerator};
use std::cell::RefCell;

pub struct Player<'a> {
    id: usize,
    main_pile: &'a RefCell<MainCardPile<'a>>,
    main_board: RefCell<Board<'a>>,
    pub hand: RefCell<BoxCardSet<'a>>,
    pub score_pile: RefCell<BoxCardSet<'a>>,
    achievements: RefCell<BoxAchievementSet<'a>>,
}

impl<'a> Player<'a> {
    pub fn new(
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
            let _main_icon = card.main_icon();
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
                    Dogma::Demand(flow) => {
                        // should filter out ineligible players
                        for player in game.players(self.id) {
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
