use crate::enums::{Color, Icon};
use crate::flow::{DemandFlow, ShareFlow};
use crate::observation::SingleAchievementView;
use counter::Counter;
use serde::Serialize;
use std::fmt::Debug;

pub enum Dogma {
    Share(ShareFlow),
    Demand(DemandFlow),
}

impl Serialize for Dogma {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Dogma::Share(_) => serializer.serialize_str("share"),
            Dogma::Demand(_) => serializer.serialize_str("demand"),
        }
    }
}

#[derive(Serialize)]
pub struct Card {
    name: String,
    age: u8,
    color: Color,
    icons: [Icon; 4],
    main_icon: Icon,
    dogmas: Vec<Dogma>,
    doc: String,
}

impl Card {
    pub fn new(
        name: String,
        age: u8,
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
            main_icon: *icons.iter().collect::<Counter<_>>().most_common()[0].0,
            dogmas,
            doc,
        }
    }
    pub fn age(&self) -> u8 {
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

impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(PartialEq, Debug, Serialize)]
pub enum SpecialAchievement {
    Universe,
    Wonder,
    World,
    // TODO: more
}

#[derive(PartialEq, Debug)]
pub enum Achievement {
    Normal(Card),
    Special(SpecialAchievement),
}

impl Achievement {
    pub fn view(&self) -> SingleAchievementView {
        match self {
            Achievement::Normal(c) => SingleAchievementView::Normal(c.age),
            Achievement::Special(s) => SingleAchievementView::Special(s),
        }
    }
}
