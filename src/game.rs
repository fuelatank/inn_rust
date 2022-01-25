
use crate::card::Card;
use crate::card::Achievement;
use crate::containers::{Addable, Removeable, Popable, CardSet};
use crate::board::Board;
use crate::card_pile::MainCardPile;

pub struct Player<'a, T: CardSet<Card>, U: Addable<Achievement> + Default> {
    game: &'a Game<'a, T, U>,
    main_board: Board,
    pub hand: T,
    pub score_pile: T,
    achievements: U
}

impl<'a, T: CardSet<Card>, U: Addable<Achievement> + Default> Player<'a, T, U> {
    fn new(game: &'a Game<'a, T, U>) -> Player<'a, T, U> {
        Player {
            game,
            main_board: Board::new(),
            hand: Default::default(),
            score_pile: Default::default(),
            achievements: Default::default()
        }
    }

    pub fn age(&self) -> u8 {
        self.main_board.highest_age()
    }

    pub fn draw(&self, age: u8) -> bool {
        transfer_first(&self.game.pile().aged(age), &self.hand)
    }

    pub fn meld(&self, card: &Card) -> bool {
        transfer_elem(&self.hand, &self.main_board.forward(), card)
    }

    pub fn tuck(&self, card: &Card) -> bool {
        transfer_elem(&self.hand, &self.main_board.backward(), card)
    }

    pub fn score(&self, card: &Card) -> bool {
        transfer_elem(&self.hand, &self.score_pile, card)
    }

    pub fn draw_and_meld(&self, age: u8) -> bool {
        transfer_first(&self.game.pile().aged(age), &self.main_board.forward())
    }

    pub fn draw_and_tuck(&self, age: u8) -> bool {
        transfer_first(&self.game.pile().aged(age), &self.main_board.backward())
    }

    pub fn draw_and_score(&self, age: u8) -> bool {
        transfer_first(&self.game.pile().aged(age), &self.score_pile)
    }

    pub fn achieve(&self, source: &impl Removeable<Achievement>, card: &Achievement) -> bool{
        transfer_elem(source, &self.achievements, card)
    }
}

pub struct Game<'a, T: CardSet<Card>, U: Addable<Achievement> + Default> {
    main_card_pile: MainCardPile,
    players: Vec<Player<'a, T, U>>,
}

fn transfer_first<T>(from: &impl Popable<T>, to: &impl Addable<T>) -> bool {
    let elem = from.pop();
    to.optional_add(elem)
}

pub fn transfer_elem<T>(from: &impl Removeable<T>, to: &impl Addable<T>, elem: &T) -> bool {
    let temp = from.remove(elem);
    to.optional_add(temp)
}

impl<'a, T: CardSet<Card>, U: Addable<Achievement> + Default> Game<'a, T, U> {
    pub fn new() -> Game<'a, T, U> {
        Game {
            main_card_pile: MainCardPile::new(),
            players: vec![]
        }
    }

    pub fn add_player(&'a self) {
        self.players.push(Player::new(self))
    }

    pub fn pile(&self) -> &MainCardPile {
        &self.main_card_pile
    }
}

mod tests {
    use super::*;
    use crate::containers::VecSet;

    #[test]
    fn create_game_player() {
        let game: Game<VecSet<Card>, VecSet<Achievement>> = Game::new();
        game.add_player();
    }
}