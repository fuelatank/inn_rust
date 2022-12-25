use crate::{
    board::Board,
    card::Achievement,
    containers::{BoxCardSet, VecSet},
    enums::{Color, Splay},
    observation::{MainPlayerView, OtherPlayerView},
};
use std::cell::{Ref, RefCell, RefMut};

pub struct Player<'c> {
    id: usize,
    main_board: RefCell<Board<'c>>,
    pub hand: RefCell<BoxCardSet<'c>>,
    pub score_pile: RefCell<BoxCardSet<'c>>,
    achievements: RefCell<VecSet<Achievement<'c>>>,
}

impl<'c> Player<'c> {
    pub fn new(
        id: usize,
        hand: BoxCardSet<'c>,
        score_pile: BoxCardSet<'c>,
        achievements: VecSet<Achievement<'c>>,
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

    pub fn with_id<T>(&self, t: T) -> (usize, T) {
        (self.id(), t)
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

    pub fn total_score(&self) -> usize {
        self.score_pile().as_iter().map(|i| i.age() as usize).sum()
    }

    pub fn achievements(&self) -> Ref<VecSet<Achievement<'c>>> {
        self.achievements.borrow()
    }

    pub fn achievements_mut(&self) -> RefMut<VecSet<Achievement<'c>>> {
        self.achievements.borrow_mut()
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
                .inner()
                .iter()
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
                .clone_inner()
                .into_iter()
                .map(|a| a.view())
                .collect(),
        }
    }
}
