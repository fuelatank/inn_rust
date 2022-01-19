
use crate::enums::{Color, Splay};
use crate::card::{Card, Achievement};

trait Addable<T> {
    fn add(&self, elem: T);

    fn optional_add(&self, elem: Option<T>) -> bool {
        // return success?
        match elem {
            Some(value) => {
                self.add(value);
                true
            }
            None => false
        }
    }
}

trait Removeable<T> {
    fn remove(&self, elem: &T) -> Option<T>; // return ownership
}

trait Popable<T> {
    fn pop(&self) -> Option<T>;
}

trait CardSet<T>: Addable<T> + Removeable<T> {}

struct CardPile {
    cards: std::collections::VecDeque<Card>
}

impl CardPile {
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

struct MainCardPile {
    piles: [CardPile; 10]
}

impl<'a> MainCardPile {
    fn aged(&'a self, age: u8) -> AgePileWrapper<'a> {
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

struct Stack {
    color: Color,
    cards: std::collections::VecDeque<Card>,
    splay: Splay
}

impl Stack {
    fn push_back(&self, card: Card) {
        self.cards.push_back(card)
    }

    fn pop_back(&self) -> Option<Card> {
        self.cards.pop_back()
    }

    fn push_front(&self, card: Card) {
        self.cards.push_front(card)
    }

    fn pop_front(&self) -> Option<Card> {
        self.cards.pop_front()
    }
}

struct Board {
    stacks: [Stack; 5]
}

struct ForwardWrapper<'a> {
    board: &'a Board
}

impl<'a> Addable<Card> for ForwardWrapper<'a> {
    fn add(&self, elem: Card) {
        self.board.meld(elem)
    }
}

struct BackwardWrapper<'a> {
    board: &'a Board
}

impl<'a> Addable<Card> for BackwardWrapper<'a> {
    fn add(&self, elem: Card) {
        self.board.tuck(elem)
    }
}

impl<'a> Board {
    fn forward(&'a self) -> ForwardWrapper<'a> {
        ForwardWrapper { board: self }
    }

    fn backward(&'a self) -> BackwardWrapper<'a> {
        BackwardWrapper { board: self }
    }

    fn meld(&self, card: Card) {
        self.stacks[card.color().as_usize()].push_front(card)
    }

    fn tuck(&self, card: Card) {
        self.stacks[card.color().as_usize()].push_back(card)
    }
}

struct Player<T: CardSet<Card>, U: Addable<Achievement>> {
    main_board: Board,
    hand: T,
    score_pile: T,
    achievements: U
}

impl<T: CardSet<Card>, U: Addable<Achievement>> Player<T, U> {
    fn draw(&self, pile: &MainCardPile, age: u8) -> bool {
        transfer_first(&pile.aged(age), &self.hand)
    }

    fn meld(&self, card: &Card) -> bool {
        transfer_elem(&self.hand, &self.main_board.forward(), card)
    }

    fn tuck(&self, card: &Card) -> bool {
        transfer_elem(&self.hand, &self.main_board.backward(), card)
    }

    fn score(&self, card: &Card) -> bool {
        transfer_elem(&self.hand, &self.score_pile, card)
    }

    fn achieve(&self, source: &impl Removeable<Achievement>, card: &Achievement) -> bool{
        transfer_elem(source, &self.achievements, card)
    }
}

struct Game<T: CardSet<Card>, U: CardSet<Achievement>> {
    main_card_pile: CardPile,
    players: Vec<Player<T, U>>,
}

fn transfer_first<T>(from: &impl Popable<T>, to: &impl Addable<T>) -> bool {
    let elem = from.pop();
    to.optional_add(elem)
}

fn transfer_elem<T>(from: &impl Removeable<T>, to: &impl Addable<T>, elem: &T) -> bool {
    let temp = from.remove(elem);
    to.optional_add(temp)
}

impl<T: CardSet<Card>, U: CardSet<Achievement>> Game<T, U> {
}