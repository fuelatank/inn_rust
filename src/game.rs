
use crate::card::Card;
use crate::card::Achievement;
use crate::containers::{Addable, Removeable, Popable, CardSet};
use crate::board::Board;
use crate::card_pile::MainCardPile;

pub struct Player<T: CardSet<Card>, U: Addable<Achievement>> {
    game: Game<T, U>,
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

pub struct Game<T: CardSet<Card>, U: Addable<Achievement>> {
    main_card_pile: MainCardPile,
    players: Vec<Player<T, U>>,
}

fn transfer_first<T>(from: &impl Popable<T>, to: &impl Addable<T>) -> bool {
    let elem = from.pop();
    to.optional_add(elem)
}

pub fn transfer_elem<T>(from: &impl Removeable<T>, to: &impl Addable<T>, elem: &T) -> bool {
    let temp = from.remove(elem);
    to.optional_add(temp)
}

impl<T: CardSet<Card>, U: CardSet<Achievement>> Game<T, U> {
    fn new() -> Game<T, U> {
        Game {
            main_card_pile: MainCardPile::new(),
            players: vec![]
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn create_game_player() {
        let game = Game::new();
    }
}