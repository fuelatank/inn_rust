use counter::Counter;
use serde::{ser::SerializeStruct, Serialize};
use strum::IntoEnumIterator;

use crate::{
    card::{Age, Card, Color, Icon, Splay},
    containers::{Addable, Removeable},
};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Default, Clone, Serialize)]
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

    fn remove(&mut self, card: &Card) -> Option<&'a Card> {
        let index = self.cards.iter().position(|&x| x == card)?;
        self.cards.remove(index)
    }

    fn insert(&mut self, card: &'a Card, index: usize) {
        self.cards.insert(index, card)
    }

    // TODO: fix mandatory unsplay etc.
    // by adding specific case for unsplay or returning a Result
    /// Splay the stack.
    ///
    /// Panics when already splayed that direction or the stack has less than two cards.
    pub fn splay(&mut self, direction: Splay) {
        assert!(self.can_splay(direction));
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

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn can_splay(&self, direction: Splay) -> bool {
        self.cards.len() >= 2 && !self.is_splayed(direction)
    }

    pub fn top_card(&self) -> Option<&'a Card> {
        match self.cards.front() {
            Some(v) => Some(*v),
            None => None,
        }
    }

    pub fn icon_count(&self) -> Counter<Icon, usize> {
        let mut counter = Counter::new();
        let mask = self.splay.mask();
        let mut card_iter = self.cards.iter();
        match card_iter.next() {
            Some(top_card) => {
                for icon in top_card.icons() {
                    counter[&icon] += 1;
                }
            }
            None => return counter,
        }
        for card in card_iter {
            for (icon, shown) in card.icons().iter().zip(mask) {
                if shown {
                    counter[icon] += 1;
                }
            }
        }
        counter
    }
}

#[derive(Debug, Default, Clone)]
pub struct Board<'a> {
    stacks: [Stack<'a>; 5],
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
        }
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

    pub fn meld(&mut self, card: &'a Card) {
        let stack = &mut self.stacks[card.color().as_usize()];
        stack.push_front(card)
    }

    pub fn tuck(&mut self, card: &'a Card) {
        self.stacks[card.color().as_usize()].push_back(card)
    }

    pub fn insert(&mut self, card: &'a Card, index: usize) {
        self.stacks[card.color().as_usize()].insert(card, index)
    }

    pub fn top_cards(&self) -> Vec<&'a Card> {
        let mut r: Vec<&Card> = Vec::new();
        for stack in self.stacks.iter() {
            if let Some(c) = stack.top_card() {
                r.push(c);
            }
        }
        r
    }

    fn highest_top_card(&self) -> Option<&'a Card> {
        let top_cards = self.top_cards();
        top_cards.into_iter().max_by_key(|card| card.age())
    }

    pub fn highest_age(&self) -> Age {
        match self.highest_top_card() {
            Some(card) => card.age(),
            None => 0,
        }
    }

    pub fn icon_count(&self) -> Counter<Icon> {
        self.stacks
            .iter()
            .map(|stack| stack.icon_count())
            .reduce(|accum, item| accum + item)
            .unwrap()
    }

    pub fn regular_icon_count(&self) -> HashMap<Icon, usize> {
        let raw_count = self.icon_count();
        Icon::iter().map(|icon| (icon, raw_count[&icon])).collect()
    }
}

impl<'a> Addable<&'a Card> for Board<'a> {
    fn add(&mut self, elem: &'a Card) {
        self.meld(elem);
        /*if self.is_forward {
            self.meld(elem)
        } else {
            self.tuck(elem)
        }*/
    }
}

impl<'a> Removeable<&'a Card, Card> for Board<'a> {
    fn remove(&mut self, param: &Card) -> Option<&'a Card> {
        let stack = self.get_stack_mut(param.color());
        stack.remove(param)
    }
}

impl<'a> Removeable<&'a Card, bool> for Stack<'a> {
    fn remove(&mut self, param: &bool) -> Option<&'a Card> {
        if *param {
            self.pop_front()
        } else {
            self.pop_back()
        }
    }
}

impl<'a> Removeable<&'a Card, usize> for Stack<'a> {
    fn remove(&mut self, param: &usize) -> Option<&'a Card> {
        self.cards.remove(*param)
    }
}

impl<'a, P> Removeable<&'a Card, (Color, P)> for Board<'a>
where
    Stack<'a>: Removeable<&'a Card, P>,
{
    fn remove(&mut self, param: &(Color, P)) -> Option<&'a Card> {
        <Stack as Removeable<&'a Card, P>>::remove(self.get_stack_mut(param.0), &param.1)
    }
}

impl<'a> Serialize for Board<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Board", 2)?;
        s.serialize_field("stacks", &self.stacks)?;
        s.serialize_field("icons", &self.regular_icon_count())?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::card::default_cards::{agriculture, archery, domestication, metalworking, tools};

    use super::*;

    #[test]
    fn board_serialization() {
        let mut board = Board::new();
        let cards = vec![
            agriculture(),
            archery(),
            domestication(),
            metalworking(),
            tools(),
        ];
        for card in cards.iter() {
            board.meld(card);
        }
        board.get_stack_mut(Color::Yellow).splay(Splay::Right);
        board.get_stack_mut(Color::Red).splay(Splay::Up);
        assert_eq!(
            serde_json::to_value(board).unwrap()["icons"],
            json!({
                "Castle": 7,
                "Clock": 0,
                "Crown": 1,
                "Factory": 0,
                "Leaf": 1,
                "Lightbulb": 3,
            })
        );
    }
}
