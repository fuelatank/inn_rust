use crate::{
    action::Action,
    card::Card,
    enums::{Color, Splay},
};

pub enum PlayerPlace {
    Hand,
    Score,
    Board,
}

pub enum Place {
    MainCardPile,
    Player(usize, PlayerPlace),
}

// in a narrow sense
pub enum Operation<'c> {
    Draw(usize, u8),
    Meld(usize, &'c Card),
    Tuck(usize, &'c Card),
    Score(usize, &'c Card),
    Splay(usize, Color, Splay),
    Return(usize, &'c Card),
    Transfer(Place, Place, &'c Card),
}

pub enum Item<'c> {
    Action(Action<'c>),
    Operation(Operation<'c>),
}

pub struct Game<'c> {
    pub initial_cards: [Vec<&'c Card>; 10],
    pub items: Vec<Item<'c>>,
}

impl<'c> Game<'c> {
    fn new(initial_cards: [Vec<&'c Card>; 10]) -> Self {
        Self {
            initial_cards,
            items: Vec::new(),
        }
    }
}

// should also have the initial arrangement etc.
pub struct Logger<'c> {
    history: Vec<Game<'c>>,
    current_game: Option<Game<'c>>,
}

impl<'c> Logger<'c> {
    pub fn new() -> Self {
        Logger {
            history: Vec::new(),
            current_game: None,
        }
    }

    pub fn act(&mut self, action: Action<'c>) {
        self.current_game
            .as_mut()
            .expect("cards not initialized")
            .items
            .push(Item::Action(action));
    }

    pub fn operate(&mut self, operation: Operation<'c>) {
        self.current_game
            .as_mut()
            .expect("cards not initialized")
            .items
            .push(Item::Operation(operation));
    }

    pub fn finish(&mut self) {
        self.history
            .push(std::mem::take(&mut self.current_game).expect("cards not initialized"));
    }

    pub fn start(&mut self, initial_cards: [Vec<&'c Card>; 10]) {
        match self.current_game {
            Some(_) => panic!("already initialized"),
            None => self.current_game = Some(Game::new(initial_cards)),
        }
    }

    pub fn history(&self) -> &[Game<'c>] {
        &self.history
    }
}
