use crate::{
    board::{Board, Stack},
    card::{Achievement, Age, Card, Color, Splay},
    containers::{Addable, BoxCardSet, CardSet, VecSet},
    game::PlayerId,
    observation::{MainPlayerView, OtherPlayerView},
};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct Player<'c> {
    id: usize,
    main_board: RwLock<Board<'c>>,
    pub hand: RwLock<BoxCardSet<'c>>,
    pub score_pile: RwLock<BoxCardSet<'c>>,
    achievements: RwLock<VecSet<Achievement<'c>>>,
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
            main_board: RwLock::new(Board::new()),
            hand: RwLock::new(hand),
            score_pile: RwLock::new(score_pile),
            achievements: RwLock::new(achievements),
        }
    }

    pub fn builder<C>() -> PlayerBuilder<'c>
    where
        C: CardSet<'c, Card> + Default + 'c + Send + Sync,
    {
        PlayerBuilder::new::<C>()
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn with_id<T>(&self, t: T) -> (usize, T) {
        (self.id(), t)
    }

    pub fn age(&self) -> Age {
        self.main_board.read().unwrap().highest_age()
    }

    pub fn hand(&self) -> RwLockReadGuard<BoxCardSet<'c>> {
        self.hand.read().unwrap()
    }

    pub fn score_pile(&self) -> RwLockReadGuard<BoxCardSet<'c>> {
        self.score_pile.read().unwrap()
    }

    pub fn board(&self) -> RwLockReadGuard<Board<'c>> {
        self.main_board.read().unwrap()
    }

    pub fn board_mut(&self) -> RwLockWriteGuard<Board<'c>> {
        self.main_board.write().unwrap()
    }

    pub fn total_score(&self) -> usize {
        self.score_pile().iter().map(|i| i.age() as usize).sum()
    }

    pub fn achievements(&self) -> RwLockReadGuard<VecSet<Achievement<'c>>> {
        self.achievements.read().unwrap()
    }

    pub fn achievements_mut(&self) -> RwLockWriteGuard<VecSet<Achievement<'c>>> {
        self.achievements.write().unwrap()
    }

    pub fn with_stack<T, F: FnOnce(&Stack<'c>) -> T>(&self, color: Color, f: F) -> T {
        f(self.main_board.read().unwrap().get_stack(color))
    }

    pub fn is_splayed(&self, color: Color, direction: Splay) -> bool {
        self.main_board.read().unwrap().is_splayed(color, direction)
    }

    pub fn can_splay(&self, color: Color, direction: Splay) -> bool {
        self.with_stack(color, |s| s.can_splay(direction))
    }

    pub fn self_view(&self) -> MainPlayerView {
        MainPlayerView {
            hand: self.hand.read().unwrap().to_vec(),
            score: self.score_pile.read().unwrap().to_vec(),
            board: self.main_board.read().unwrap(), /* what if it's mut borrowed? */
            achievements: self
                .achievements
                .read()
                .unwrap()
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
            board: self.main_board.read().unwrap(), /* what if it's mut borrowed? */
            achievements: self
                .achievements
                .read()
                .unwrap()
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
        C: CardSet<'c, Card> + Default + 'c + Send + Sync,
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
            main_board: RwLock::new(self.main_board),
            hand: RwLock::new(self.hand),
            score_pile: RwLock::new(self.score_pile),
            achievements: RwLock::new(self.achievements),
        }
    }
}

impl<'c> Default for PlayerBuilder<'c> {
    fn default() -> Self {
        PlayerBuilder::new::<VecSet<_>>()
    }
}
