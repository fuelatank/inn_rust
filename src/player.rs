use crate::{
    board::{Board, Stack},
    card::{Achievement, Card},
    containers::{Addable, BoxCardSet, CardSet, VecSet},
    enums::{Color, Splay},
    game::PlayerId,
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

    pub fn board(&self) -> Ref<Board<'c>> {
        self.main_board.borrow()
    }

    pub fn board_mut(&self) -> RefMut<Board<'c>> {
        self.main_board.borrow_mut()
    }

    pub fn total_score(&self) -> usize {
        self.score_pile().iter().map(|i| i.age() as usize).sum()
    }

    pub fn achievements(&self) -> Ref<VecSet<Achievement<'c>>> {
        self.achievements.borrow()
    }

    pub fn achievements_mut(&self) -> RefMut<VecSet<Achievement<'c>>> {
        self.achievements.borrow_mut()
    }

    pub fn stack(&self, color: Color) -> Ref<Stack<'c>> {
        Ref::map(self.main_board.borrow(), |board| board.get_stack(color))
    }

    pub fn is_splayed(&self, color: Color, direction: Splay) -> bool {
        self.main_board.borrow().is_splayed(color, direction)
    }

    pub fn can_splay(&self, color: Color, direction: Splay) -> bool {
        self.stack(color).can_splay(direction)
    }

    pub fn self_view(&self) -> MainPlayerView {
        MainPlayerView {
            hand: self.hand.borrow().to_vec(),
            score: self.score_pile.borrow().to_vec(),
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
            hand: self.hand().to_vec().into_iter().map(|c| c.age()).collect(),
            score: self
                .score_pile()
                .to_vec()
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

pub struct PlayerBuilder<'c> {
    main_board: Board<'c>,
    hand: BoxCardSet<'c>,
    score_pile: BoxCardSet<'c>,
    achievements: VecSet<Achievement<'c>>,
}

impl<'c> PlayerBuilder<'c> {
    pub fn new<C>() -> PlayerBuilder<'c>
    where
        C: CardSet<'c, Card> + Default + 'c,
    {
        PlayerBuilder {
            main_board: Board::new(),
            hand: Box::<C>::default(),
            score_pile: Box::<C>::default(),
            achievements: VecSet::default(),
        }
    }

    pub fn hand(mut self, hand: Vec<&'c Card>) -> PlayerBuilder<'c> {
        for card in hand {
            self.hand.add(card);
        }
        self
    }

    pub fn score(mut self, score: Vec<&'c Card>) -> PlayerBuilder<'c> {
        for card in score {
            self.score_pile.add(card);
        }
        self
    }

    pub fn board(mut self, cards: Vec<&'c Card>) -> PlayerBuilder<'c> {
        for card in cards {
            self.main_board.tuck(card);
        }
        self
    }

    pub fn splay(mut self, color: Color, direction: Splay) -> PlayerBuilder<'c> {
        self.main_board.get_stack_mut(color).splay(direction);
        self
    }

    pub fn achievements(mut self, achievements: Vec<Achievement<'c>>) -> PlayerBuilder<'c> {
        for achievement in achievements {
            self.achievements.add(achievement);
        }
        self
    }

    pub fn build(self, id: PlayerId) -> Player<'c> {
        Player {
            id,
            main_board: RefCell::new(self.main_board),
            hand: RefCell::new(self.hand),
            score_pile: RefCell::new(self.score_pile),
            achievements: RefCell::new(self.achievements),
        }
    }
}
