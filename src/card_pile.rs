use crate::{
    card::{Achievement, Card, SpecialAchievement},
    containers::{Addable, Removeable, VecSet},
    observation::SingleAchievementView,
};
use std::collections::VecDeque;

pub type CardOrder<'c> = [Vec<&'c Card>; 10];

#[derive(Clone)]
struct CardPile<'a> {
    cards: VecDeque<&'a Card>,
}

impl<'a> CardPile<'a> {
    fn new() -> CardPile<'a> {
        CardPile {
            cards: VecDeque::new(),
        }
    }

    fn len(&self) -> usize {
        self.cards.len()
    }
}

impl<'a> Addable<&'a Card> for CardPile<'a> {
    fn add(&mut self, card: &'a Card) {
        self.cards.push_back(card)
    }
}

impl<'a> Removeable<&'a Card, ()> for CardPile<'a> {
    fn remove(&mut self, _: &()) -> Option<&'a Card> {
        self.cards.pop_front()
    }
}

pub struct MainCardPile<'a> {
    piles: [CardPile<'a>; 10],
    achievements: VecSet<Achievement<'a>>,
}

impl<'a> MainCardPile<'a> {
    pub fn empty() -> MainCardPile<'a> {
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
                CardPile::new(),
            ],
            achievements: Default::default(),
        }
    }

    pub fn new(
        cards: Vec<&'a Card>,
        achievements: impl IntoIterator<Item = Achievement<'a>>,
    ) -> MainCardPile<'a> {
        let mut pile = MainCardPile::empty();
        for card in cards {
            pile.add(card);
        }
        for achievement in achievements {
            pile.achievements.add(achievement)
        }
        pile
    }

    pub fn new_init(
        cards: Vec<&'a Card>,
        special_achievements: Vec<SpecialAchievement>,
    ) -> MainCardPile<'a> {
        let mut pile = MainCardPile::new(
            cards,
            special_achievements.into_iter().map(Achievement::Special),
        );

        // pick one (if exists) card of each of the first 9 ages as achievement
        for age in pile.piles.iter_mut().take(9) {
            if let Some(card) = age.remove(&()) {
                pile.achievements.add(Achievement::Normal(card));
            }
        }

        pile
    }

    fn pop_age(&mut self, age: u8) -> Option<&'a Card> {
        if age >= 11 {
            return None;
        }
        let index = if age == 0 { 0 } else { age - 1 };
        match self.piles[index as usize].remove(&()) {
            Some(card) => Some(card),
            None => self.pop_age(age + 1),
        }
    }

    pub fn contents(&self) -> CardOrder<'a> {
        self.piles
            .clone()
            .map(|pile| pile.cards.iter().map(Clone::clone).collect())
    }

    pub fn view(&self) -> [usize; 10] {
        [
            self.piles[0].len(),
            self.piles[1].len(),
            self.piles[2].len(),
            self.piles[3].len(),
            self.piles[4].len(),
            self.piles[5].len(),
            self.piles[6].len(),
            self.piles[7].len(),
            self.piles[8].len(),
            self.piles[9].len(),
        ]
    }

    pub fn has_achievement(&self, view: &SingleAchievementView) -> bool {
        self.achievements.inner().iter().any(|a| a == view)
    }
}

impl<'a> Addable<&'a Card> for MainCardPile<'a> {
    fn add(&mut self, card: &'a Card) {
        let age = card.age();
        self.piles[(age - 1) as usize].add(card)
    }
}

impl<'a> Removeable<&'a Card, u8> for MainCardPile<'a> {
    fn remove(&mut self, age: &u8) -> Option<&'a Card> {
        self.pop_age(*age)
    }
}

impl<'a> Removeable<Achievement<'a>, SingleAchievementView> for MainCardPile<'a> {
    fn remove(&mut self, achievement: &SingleAchievementView) -> Option<Achievement<'a>> {
        self.achievements.try_remove(|a| a == achievement)
    }
}
