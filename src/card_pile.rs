
use std::collections::VecDeque;
use crate::card::Card;
use crate::containers::{Addable, Popable};

struct CardPile {
    cards: VecDeque<Card>
}

impl CardPile {
    fn new() -> CardPile {
        CardPile {
            cards: VecDeque::new()
        }
    }

    fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl Addable<Card> for CardPile {
    fn add(&self, card: Card) {
        self.cards.push_back(card)
    }
}

impl Popable<Card> for CardPile {
    fn pop(&self) -> Option<Card> {
        self.cards.pop_front()
    }
}

struct AgePileWrapper<'a> {
    main_pile: &'a MainCardPile,
    age: u8
}

impl<'a> Popable<Card> for AgePileWrapper<'a> {
    fn pop(&self) -> Option<Card> {
        self.main_pile.pop_age(self.age)
    }
}

pub struct MainCardPile {
    piles: [CardPile; 10]
}

impl<'a> MainCardPile {
    pub fn new() -> MainCardPile {
        MainCardPile {
            piles: [
                CardPile::new(),
                CardPile::new(),
                CardPile::new(),
                CardPile::new(),
                CardPile::new(),
                CardPile::new(),
                CardPile::new(),
                CardPile::new(),
                CardPile::new(),
                CardPile::new()
            ]
        }
    }

    pub fn aged(&'a self, age: u8) -> AgePileWrapper<'a> {
        AgePileWrapper { main_pile: self, age }
    }

    fn pop_age(&self, age: u8) -> Option<Card> {
        if age >= 11 || age == 0 {
            return None;
        }
        match self.piles[(age - 1) as usize].pop() {
            Some(card) => Some(card),
            None => self.pop_age(age + 1)
        }
    }
}

impl Addable<Card> for MainCardPile {
    fn add(&self, card: Card) {
        let age = card.age();
        self.piles[(age - 1) as usize].add(card);
    }
}