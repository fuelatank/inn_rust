use crate::board::Board;
use crate::card::{Card, Dogma};
use crate::card_pile::MainCardPile;
use crate::containers::{transfer, BoxAchievementSet, BoxCardSet};
use crate::enums::{Color, Splay};
use crate::error::InnResult;
use crate::flow::FlowState;
use crate::game::{Players, RcCell};
use crate::logger::{Logger, Place, RemoveParam};
use crate::observation::{MainPlayerView, OtherPlayerView};
use generator::Gn;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Player<'c> {
    id: usize,
    logger: RcCell<Logger<'c>>,
    main_pile: RcCell<MainCardPile<'c>>,
    main_board: RefCell<Board<'c>>,
    pub hand: RefCell<BoxCardSet<'c>>,
    pub score_pile: RefCell<BoxCardSet<'c>>,
    achievements: RefCell<BoxAchievementSet<'c>>,
}

impl<'c> Player<'c> {
    pub fn new(
        id: usize,
        logger: RcCell<Logger<'c>>,
        main_pile: RcCell<MainCardPile<'c>>,
        hand: BoxCardSet<'c>,
        score_pile: BoxCardSet<'c>,
        achievements: BoxAchievementSet<'c>,
    ) -> Player<'c> {
        Player {
            id,
            logger,
            main_pile,
            main_board: RefCell::new(Board::new()),
            hand: RefCell::new(hand),
            score_pile: RefCell::new(score_pile),
            achievements: RefCell::new(achievements),
        }
    }

    pub fn id(&self) -> usize {
        self.id
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

    pub fn execute<'g>(&'g self, card: &'c Card, game: &'g Players<'c>) -> FlowState<'c, 'g> {
        unimplemented!()
        /*Gn::new_scoped_local(move |mut s| {
            let _main_icon = card.main_icon();
            for dogma in card.dogmas() {
                match dogma {
                    DogmaOld::Share(flow) => {
                        // should filter out ineligible players
                        for player in game.players_from(self.id) {
                            let mut gen = flow(player, game);

                            // s.yield_from(gen); but with or(card)
                            let mut state = gen.resume();
                            while let Some(st) = state {
                                let a = s.yield_(st.or(card)).expect("Generator got None");
                                gen.set_para(a);
                                state = gen.resume();
                            }
                        }
                    }
                    DogmaOld::Demand(flow) => {
                        // should filter out ineligible players
                        for player in game.players_from(self.id).skip(1) {
                            let mut gen = flow(self, player, game);
                            // s.yield_from(gen); but with or(card)
                            let mut state = gen.resume();
                            while let Some(st) = state {
                                let a = s.yield_(st.or(card)).expect("Generator got None");
                                gen.set_para(a);
                                state = gen.resume();
                            }
                        }
                    }
                }
            }
            generator::done!()
        })*/
    }

    pub fn self_view(&self) -> MainPlayerView {
        MainPlayerView {
            hand: self.hand.borrow().as_vec(),
            score: self.score_pile.borrow().as_vec(),
            board: self.main_board.borrow(), /* what if it's mut borrowed? */
            achievements: self
                .achievements
                .borrow()
                .as_vec()
                .into_iter()
                .map(|a| a.view())
                .collect(),
        }
    }

    pub fn other_view(&self) -> OtherPlayerView {
        OtherPlayerView {
            hand: self
                .hand
                .borrow()
                .as_vec()
                .into_iter()
                .map(|c| c.age())
                .collect(),
            score: self
                .score_pile
                .borrow()
                .as_vec()
                .into_iter()
                .map(|c| c.age())
                .collect(),
            board: self.main_board.borrow(), /* what if it's mut borrowed? */
            achievements: self
                .achievements
                .borrow()
                .as_vec()
                .into_iter()
                .map(|a| a.view())
                .collect(),
        }
    }
}
