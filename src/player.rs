use crate::board::Board;
use crate::containers::{BoxAchievementSet, BoxCardSet};
use crate::enums::{Color, Splay};
use crate::observation::{MainPlayerView, OtherPlayerView};
use std::cell::{Ref, RefCell};

pub struct Player<'c> {
    id: usize,
    main_board: RefCell<Board<'c>>,
    pub hand: RefCell<BoxCardSet<'c>>,
    pub score_pile: RefCell<BoxCardSet<'c>>,
    achievements: RefCell<BoxAchievementSet<'c>>,
}

impl<'c> Player<'c> {
    pub fn new(
        id: usize,
        hand: BoxCardSet<'c>,
        score_pile: BoxCardSet<'c>,
        achievements: BoxAchievementSet<'c>,
    ) -> Player<'c> {
        Player {
            id,
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

    pub fn hand(&self) -> Ref<BoxCardSet<'c>> {
        self.hand.borrow()
    }

    pub fn score_pile(&self) -> Ref<BoxCardSet<'c>> {
        self.score_pile.borrow()
    }

    pub fn board(&self) -> &RefCell<Board<'c>> {
        &self.main_board
    }

    pub fn is_splayed(&self, color: Color, direction: Splay) -> bool {
        self.main_board.borrow().is_splayed(color, direction)
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
            hand: self.hand().as_vec().into_iter().map(|c| c.age()).collect(),
            score: self
                .score_pile()
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
