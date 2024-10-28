use rand::{seq::SliceRandom, thread_rng};

use crate::{
    card::{Achievement, Age, Card, SpecialAchievement},
    containers::{Addable, Removeable, VecSet},
    observation::SingleAchievementView,
};
use std::{array, collections::VecDeque};

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
        MainCardPileBuilder::new()
            .draw_deck(cards)
            .shuffled()
            .pick_normal()
            .special_achievements(special_achievements)
            .build()
    }

    pub fn builder() -> MainCardPileBuilder<'a> {
        MainCardPileBuilder::new()
    }

    fn pop_age(&mut self, age: Age) -> Option<&'a Card> {
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

impl<'a> Removeable<&'a Card, Age> for MainCardPile<'a> {
    fn remove(&mut self, age: &Age) -> Option<&'a Card> {
        self.pop_age(*age)
    }
}

impl<'a> Removeable<Achievement<'a>, SingleAchievementView> for MainCardPile<'a> {
    fn remove(&mut self, achievement: &SingleAchievementView) -> Option<Achievement<'a>> {
        self.achievements.try_remove(|a| a == achievement)
    }
}

#[derive(Default)]
pub struct MainCardPileBuilder<'a> {
    piles: Vec<&'a Card>,
    achievements: Vec<Achievement<'a>>,
    pick_normal_after_init: bool,
}

impl<'a> MainCardPileBuilder<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn draw_deck(mut self, cards: Vec<&'a Card>) -> Self {
        self.piles = cards;
        self
    }

    pub fn achievements(mut self, achievements: Vec<Achievement<'a>>) -> Self {
        self.achievements = achievements;
        self
    }

    pub fn special_achievements(mut self, achievements: Vec<SpecialAchievement>) -> Self {
        self.achievements
            .extend(achievements.into_iter().map(Achievement::Special));
        self
    }

    pub fn shuffled(mut self) -> Self {
        self.piles.shuffle(&mut thread_rng());
        self
    }

    /// Draw cards from the deck as normal achievements
    pub fn pick_normal(mut self) -> Self {
        self.pick_normal_after_init = true;
        self
    }

    pub fn build(self) -> MainCardPile<'a> {
        let mut pile = MainCardPile::new(self.piles, self.achievements);
        if self.pick_normal_after_init {
            // pick one (if exists) card of each of the first 9 ages as achievement
            for age in pile.piles.iter_mut().take(9) {
                if let Some(card) = age.remove(&()) {
                    pile.achievements.add(Achievement::Normal(card));
                }
            }
        }
        pile
    }
}

pub fn split_cards<'a>(cards: impl IntoIterator<Item = &'a Card>) -> CardOrder<'a> {
    let mut ordered = array::from_fn(|_| Vec::new());
    for card in cards {
        ordered[<u8 as Into<usize>>::into(card.age()) - 1].push(card);
    }
    ordered
}
