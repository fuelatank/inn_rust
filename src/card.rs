use crate::{
    card_attrs::{Color, Icon, Age},
    flow::Dogma,
    observation::SingleAchievementView,
};
use counter::Counter;
use serde::Serialize;
use std::{fmt::Debug, hash::Hash};
use strum_macros::EnumIter;

fn main_icon(icons: [Icon; 4]) -> Icon {
    *icons.iter().collect::<Counter<_>>().most_common()[0].0
}

#[derive(Serialize)]
pub struct Card {
    name: String,
    age: Age,
    color: Color,
    icons: [Icon; 4],
    main_icon: Icon,
    dogmas: Vec<Dogma>,
    doc: String,
}

impl Card {
    pub fn new_noop(name: String, age: Age, color: Color, icons: [Icon; 4]) -> Card {
        Card {
            name,
            age,
            color,
            icons,
            main_icon: main_icon(icons),
            dogmas: Vec::new(),
            doc: String::new(),
        }
    }

    pub fn new(
        name: String,
        age: Age,
        color: Color,
        icons: [Icon; 4],
        dogmas: Vec<Dogma>,
        doc: String,
    ) -> Card {
        Card {
            name,
            age,
            color,
            icons,
            main_icon: main_icon(icons),
            dogmas,
            doc,
        }
    }

    pub fn age(&self) -> Age {
        self.age
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn contains(&self, icon: Icon) -> bool {
        self.icons.contains(&icon)
    }

    pub fn doc(&self) -> &String {
        &self.doc
    }

    pub fn dogmas(&self) -> &[Dogma] {
        &self.dogmas
    }

    pub fn icons(&self) -> [Icon; 4] {
        self.icons
    }

    pub fn main_icon(&self) -> Icon {
        self.main_icon
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Card) -> bool {
        self.name == other.name
    }
}

impl Eq for Card {}

impl Hash for Card {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(PartialEq, Debug, Serialize, EnumIter, Clone, Copy)]
pub enum SpecialAchievement {
    Monument,
    Empire,
    World,
    Wonder,
    Universe,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Achievement<'a> {
    Normal(&'a Card),
    Special(SpecialAchievement),
}

impl<'a> Achievement<'a> {
    pub fn view(&self) -> SingleAchievementView {
        match self {
            Achievement::Normal(c) => SingleAchievementView::Normal(c.age),
            Achievement::Special(s) => SingleAchievementView::Special(*s),
        }
    }
}
