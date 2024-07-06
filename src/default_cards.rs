use convert_case::{Case, Casing};

use crate::{
    card::Card,
    card_attrs::{Color::*, Icon::*},
    dogma_fn,
};

macro_rules! card_decl {
    ($name:ident, $age:expr, $color:expr, $icons:expr, $doc:expr) => {
        pub fn $name() -> Card {
            Card::new(
                stringify!($name)
                    .from_case(Case::Snake)
                    .to_case(Case::Title),
                $age,
                $color,
                $icons,
                dogma_fn::$name(),
                $doc.into(),
            )
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

    tools, 1, Blue, [Empty, Lightbulb, Lightbulb, Castle],
    "You may return three cards from your hand. If you do, draw and meld a 3.\n\
    You may return a 3 from your hand. If you do, draw three 1.";

    archery, 1, Red, [Castle, Lightbulb, Empty, Castle],
    "I demand you draw a 1, then transfer the highest card in your hand to my hand!";

    metalworking, 1, Red, [Castle, Castle, Empty, Castle],
    "Draw and reveal a 1. If it has a [Castle], score it and repeat this dogma effect. \
    Otherwise, keep it.";

    oars, 1, Red, [Castle, Crown, Empty, Castle],
    "I demand you transfer a card with a [Crown] from your hand to my score pile! \
    If you do, draw a 1.\n\
    If no cards were transferred due to this demand, draw a 1.";

    clothing, 1, Green, [Empty, Crown, Leaf, Leaf],
    "Meld a card from your hand of different color from any card on your board.\n\
    Draw and score a 1 for each color present on your board not present \
    on any other player’s board.";

    agriculture, 1, Yellow, [Empty, Leaf, Leaf, Leaf],
    "You may return a card from your hand. \
    If you do, draw and score a card of value one higher than the card you returned.";

    domestication, 1, Yellow, [Castle, Crown, Empty, Castle],
    "Meld the lowest card in your hand. Draw a 1.";

    masonry, 1, Yellow, [Castle, Empty, Castle, Castle],
    "You may meld any number of cards from your hand, each with a [Castle]. \
    If you melded four or more cards, claim the Monument achievement.";

    city_states, 1, Purple, [Empty, Crown, Crown, Castle],
    "I demand you transfer a top card with a [Castle] from your board to my board \
    if you have at least four [Castle] icons on your board! If you do, draw a 1!";

    code_of_laws, 1, Purple, [Empty, Crown, Crown, Leaf],
    "You may return a card from your hand. \
    If you do, draw and score a card of value one higher than the card you returned.";

    mysticism, 1, Purple, [Empty, Castle, Castle, Castle],
    "Draw a 1. If it is the same color as any card on your board, \
    meld it and draw a 1.";

    monotheism, 2, Purple, [Empty, Castle, Castle, Castle],
    "I demand you transfer a top card on your board of a different color from \
    any card on my board to my score pile! If you do, draw and tuck a 1!\n\
    Draw and tuck a 1.";

    philosophy, 2, Purple, [Empty, Lightbulb, Lightbulb, Lightbulb],
    "You may splay left any one color of your cards.\n\
    You may score a card from your hand.";

    optics, 3, Red, [Crown, Crown, Crown, Empty],
    "Draw and meld a 3. If it has a [Crown], draw and score a 4. Otherwise, \
    transfer a card from your score pile to the score pile of an opponent \
    with fewer points than you.";

    anatomy, 4, Yellow, [Leaf, Leaf, Leaf, Empty],
    "I demand you return a card from your score pile! If you do, \
    return a top card of equal value from your board!";

    enterprise, 4, Purple, [Empty, Crown, Crown, Crown],
    "I demand you transfer a top non-purple card with a [Crown] from your board \
    to my board! If you do, draw and meld a 4!\n\
    You may splay your green cards right.";

    reformation, 4, Purple, [Leaf, Leaf, Empty, Leaf],
    "You may tuck a card from your hand for every two [Leaf] icons on your board.\n\
    You may splay your yellow or purple cards right.";

    computers, 9, Blue, [Clock, Empty, Clock, Factory],
    "You may splay your red cards or your green cards up.\n\
    Draw and meld a 10, then execute its non-demand dogma effects for yourself only.";
}
