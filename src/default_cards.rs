use crate::{
    card::Card,
    dogma_fn::*,
    enums::{Color::*, Icon::*},
};

pub fn default_cards() -> Vec<Card> {
    vec![
        Card::new("Pottery".to_owned(), 1, Blue, [Empty, Leaf, Leaf, Leaf], pottery(), "You may return up to three cards from your hand. If you returned any cards, draw and score a card of value equal to the number of cards you returned.".to_owned()),
        Card::new("Tools".to_owned(), 1, Blue, [Empty, Lightblub, Lightblub, Castle], tools(), "You may return three cards from your hand. If you do, draw and meld a 3.\nYou may return a 3 from your hand. If you do, draw three 1.".to_owned()),
        Card::new("Archery".to_owned(), 1, Red, [Castle, Lightblub, Empty, Castle], archery(), "I demand you draw a 1, then transfer the highest card in your hand to my hand!".to_owned()),
        Card::new("Oars".to_owned(), 1, Red, [Castle, Crown, Empty, Castle], oars(), "I demand you transfer a card with a [Crown] from your hand to my score pile! If you do, draw a 1.\nIf no cards were transferred due to this demand, draw a 1.".to_owned()),
        Card::new("Agriculture".to_owned(), 1, Yellow, [Empty, Leaf, Leaf, Leaf], agriculture(), "You may return a card from your hand. If you do, draw and score a card of value one higher than the card you returned.".to_owned()),
        Card::new("Code of Laws".to_owned(), 1, Purple, [Empty, Crown, Crown, Leaf], code_of_laws(), "You may tuck a card from your hand of the same color as any card on your board. If you do, you may splay that color of your cards left.".to_owned()),
        Card::new("Monotheism".to_owned(), 2, Purple, [Empty, Castle, Castle, Castle], monotheism(), "I demand you transfer a top card on your board of a different color from any card on my board to my score pile! If you do, draw and tuck a 1!\nDraw and tuck a 1.".to_owned()),
        Card::new("Philosophy".to_owned(), 2, Purple, [Empty, Lightblub, Lightblub, Lightblub], philosophy(), "You may splay left any one color of your cards.\nYou may score a card from your hand.".to_owned()),
        Card::new("Optics".to_owned(), 3, Red, [Crown, Crown, Crown, Empty], optics(), "Draw and meld a 3. If it has a [Crown], draw and score a 4. Otherwise, transfer a card from your score pile to the score pile of an opponent with fewer points than you.".to_owned()),
    ]
}
