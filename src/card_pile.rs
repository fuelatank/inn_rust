
use std::collections::VecDeque;
use crate::card::Card;
use crate::containers::{Addable, Removeable};

struct CardPile<'a> {
    cards: VecDeque<&'a Card>
}

impl<'a> CardPile<'a> {
    fn new() -> CardPile<'a> {
        CardPile {
            cards: VecDeque::new()
        }
    }
}

impl<'a> Addable<'a, Card> for CardPile<'a> {
    fn add(&mut self, card: &'a Card) {
        self.cards.push_back(card)
    }
}

impl<'a> Removeable<'a, Card, ()> for CardPile<'a> {
    fn remove(&mut self, _: &()) -> Option<&'a Card> {
        self.cards.pop_front()
    }
}

pub struct MainCardPile<'a> {
    piles: [CardPile<'a>; 10],
}

impl<'a> MainCardPile<'a> {
    pub fn new() -> MainCardPile<'a> {
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
            ],
        }
    }

    fn pop_age(&mut self, age: u8) -> Option<&'a Card> {
        if age >= 11 || age == 0 {
            return None;
        }
        match self.piles[(age - 1) as usize].remove(&()) {
            Some(card) => Some(card),
            None => self.pop_age(age + 1)
        }
    }
}

impl<'a> Addable<'a, Card> for MainCardPile<'a> {
    fn add(&mut self, card: &'a Card) {
        let age = card.age();
        self.piles[(age - 1) as usize].add(card)
    }
}

impl<'a> Removeable<'a, Card, u8> for MainCardPile<'a> {
    fn remove(&mut self, age: &'a u8) -> Option<&'a Card> {
        self.pop_age(*age)
    }
}