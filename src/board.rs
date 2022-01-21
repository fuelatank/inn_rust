
use std::collections::VecDeque;
use crate::enums::{Splay};
use crate::card::Card;
use crate::containers::Addable;

struct Stack {
    cards: VecDeque<Card>,
    splay: Splay
}

impl Stack {
    fn new() -> Stack {
        Stack {
            cards: VecDeque::new(),
            splay: Splay::NoSplay
        }
    }

    fn push_back(&self, card: Card) {
        self.cards.push_back(card)
    }

    fn pop_back(&self) -> Option<Card> {
        self.cards.pop_back()
    }

    fn push_front(&self, card: Card) {
        self.cards.push_front(card)
    }

    fn pop_front(&self) -> Option<Card> {
        self.cards.pop_front()
    }
}

pub struct Board {
    stacks: [Stack; 5]
}

struct ForwardWrapper<'a> {
    board: &'a Board
}

impl<'a> Addable<Card> for ForwardWrapper<'a> {
    fn add(&self, elem: Card) {
        self.board.meld(elem)
    }
}

struct BackwardWrapper<'a> {
    board: &'a Board
}

impl<'a> Addable<Card> for BackwardWrapper<'a> {
    fn add(&self, elem: Card) {
        self.board.tuck(elem)
    }
}

impl<'a> Board {
    pub fn new() -> Board {
        Board {
            stacks: [
                Stack::new(),
                Stack::new(),
                Stack::new(),
                Stack::new(),
                Stack::new()
            ]
        }
    }

    pub fn forward(&'a self) -> ForwardWrapper<'a> {
        ForwardWrapper { board: self }
    }

    pub fn backward(&'a self) -> BackwardWrapper<'a> {
        BackwardWrapper { board: self }
    }

    fn meld(&self, card: Card) {
        self.stacks[card.color().as_usize()].push_front(card)
    }

    fn tuck(&self, card: Card) {
        self.stacks[card.color().as_usize()].push_back(card)
    }
}