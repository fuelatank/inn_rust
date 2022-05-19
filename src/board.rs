use crate::card::Card;
use crate::containers::Addable;
use crate::enums::{Color, Splay};
use std::collections::VecDeque;

pub struct Stack<'a> {
    cards: VecDeque<&'a Card>,
    splay: Splay,
}

impl<'a> Stack<'a> {
    fn new() -> Stack<'a> {
        Stack {
            cards: VecDeque::new(),
            splay: Splay::NoSplay,
        }
    }

    fn push_back(&mut self, card: &'a Card) {
        self.cards.push_back(card)
    }

    fn pop_back(&mut self) -> Option<&'a Card> {
        self.cards.pop_back()
    }

    fn push_front(&mut self, card: &'a Card) {
        self.cards.push_front(card)
    }

    fn pop_front(&mut self) -> Option<&'a Card> {
        self.cards.pop_front()
    }

    pub fn splay(&mut self, direction: Splay) {
        assert_ne!(self.splay, direction);
        self.splay = direction;
    }

    pub fn is_splayed(&self, direction: Splay) -> bool {
        self.splay == direction
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn contains(&self, card: &'a Card) -> bool {
        self.cards.contains(&card)
    }

    fn top_card(&self) -> Option<&'a Card> {
        match self.cards.front() {
            Some(v) => Some(*v),
            None => None,
        }
    }
}

pub struct Board<'a> {
    stacks: [Stack<'a>; 5],
    is_forward: bool,
}

impl<'a> Board<'a> {
    pub fn new() -> Board<'a> {
        Board {
            stacks: [
                Stack::new(),
                Stack::new(),
                Stack::new(),
                Stack::new(),
                Stack::new(),
            ],
            is_forward: true,
        }
    }

    pub fn forward(&mut self) {
        self.is_forward = true;
    }

    pub fn backward(&mut self) {
        self.is_forward = false
    }

    pub fn get_stack(&self, color: Color) -> &Stack<'a> {
        &self.stacks[color.as_usize()]
    }

    pub fn get_stack_mut(&mut self, color: Color) -> &mut Stack<'a> {
        &mut self.stacks[color.as_usize()]
    }

    pub fn is_splayed(&self, color: Color, direction: Splay) -> bool {
        self.stacks[color.as_usize()].is_splayed(direction)
    }

    pub fn contains(&self, card: &'a Card) -> bool {
        self.stacks[card.color().as_usize()].contains(card)
    }

    fn meld(&mut self, card: &'a Card) {
        let stack = &mut self.stacks[card.color().as_usize()];
        stack.push_front(card)
    }

    fn tuck(&mut self, card: &'a Card) {
        self.stacks[card.color().as_usize()].push_back(card)
    }

    fn top_cards(&self) -> Vec<&Card> {
        let mut r: Vec<&Card> = Vec::new();
        for stack in self.stacks.iter() {
            match stack.top_card() {
                Some(c) => r.push(c),
                None => {}
            }
        }
        r
    }

    fn highest_top_card(&self) -> Option<&Card> {
        let top_cards = self.top_cards();
        top_cards.into_iter().max_by_key(|card| card.age())
    }

    pub fn highest_age(&self) -> u8 {
        match self.highest_top_card() {
            Some(card) => card.age(),
            None => 0,
        }
    }
}

impl<'a> Addable<'a, Card> for Board<'a> {
    fn add(&mut self, elem: &'a Card) {
        if self.is_forward {
            self.meld(elem)
        } else {
            self.tuck(elem)
        }
    }
}
