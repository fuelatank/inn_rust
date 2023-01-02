use convert_case::{Case, Casing};

use crate::{
    card::Card,
    dogma_fn,
    enums::{Color::*, Icon::*},
};

macro_rules! card_decl {
    ($name:ident, $age:expr, $color:expr, $icons:expr, $doc:expr) => {
        pub fn $name() -> Card {
            Card::new(stringify!($name).from_case(Case::Snake).to_case(Case::Title), $age, $color, $icons, dogma_fn::$name(), $doc.into())
        }
    };
}

macro_rules! card_decls {
    ($($name:ident, $age:expr, $color:expr, $icons:expr, $doc:expr;)+) => {
        $(
            card_decl! {
                $name, $age, $color, $icons, $doc
            }
        )+
        pub fn default_cards() -> Vec<Card> {
            vec![$($name(),)+]
        }
    };
}

card_decls! {
    pottery, 1, Blue, [Empty, Leaf, Leaf, Leaf],
    "You may return up to three cards from your hand. If you returned any cards, \
    draw and score a card of value equal to the number of cards you returned.";

    tools, 1, Blue, [Empty, Lightblub, Lightblub, Castle],
    "You may return three cards from your hand. If you do, draw and meld a 3.\n\
    You may return a 3 from your hand. If you do, draw three 1.";

    archery, 1, Red, [Castle, Lightblub, Empty, Castle],
    "I demand you draw a 1, then transfer the highest card in your hand to my hand!";

    oars, 1, Red, [Castle, Crown, Empty, Castle],
    "I demand you transfer a card with a [Crown] from your hand to my score pile! \
    If you do, draw a 1.\n\
    If no cards were transferred due to this demand, draw a 1.";

    agriculture, 1, Yellow, [Empty, Leaf, Leaf, Leaf],
    "You may return a card from your hand. \
    If you do, draw and score a card of value one higher than the card you returned.";

    code_of_laws, 1, Purple, [Empty, Crown, Crown, Leaf],
    "You may return a card from your hand. \
    If you do, draw and score a card of value one higher than the card you returned.";

    monotheism, 2, Purple, [Empty, Castle, Castle, Castle],
    "I demand you transfer a top card on your board of a different color from \
    any card on my board to my score pile! If you do, draw and tuck a 1!\n\
    Draw and tuck a 1.";

    philosophy, 2, Purple, [Empty, Lightblub, Lightblub, Lightblub],
    "You may splay left any one color of your cards.\n\
    You may score a card from your hand.";

    optics, 3, Red, [Crown, Crown, Crown, Empty],
    "Draw and meld a 3. If it has a [Crown], draw and score a 4. Otherwise, \
    transfer a card from your score pile to the score pile of an opponent \
    with fewer points than you.";
}