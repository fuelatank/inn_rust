use crate::{
    action::Action,
    card::Card,
    card_pile::CardOrder,
    enums::{Color, Splay},
    game::PlayerId,
    structure::Place,
};

pub enum Operation<'c> {
    Splay(PlayerId, Color, Splay),
    Transfer(Place, Place, &'c Card),
}

pub enum Item<'c> {
    Action(Action),
    Operation(Operation<'c>),
}

pub struct Game<'c> {
    pub initial_cards: CardOrder<'c>,
    pub items: Vec<Item<'c>>,
}

impl<'c> Game<'c> {
    fn new(initial_cards: CardOrder<'c>) -> Self {
        Self {
            initial_cards,
            items: Vec::new(),
        }
    }
}

#[derive(Default)]
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

    pub fn log(&mut self, item: Item<'c>) {
        self.current_game
            .as_mut()
            .expect("cards not initialized")
            .items
            .push(item);
    }

    pub fn act(&mut self, action: Action) {
        self.log(Item::Action(action));
    }

    pub fn operate(&mut self, operation: Operation<'c>) {
        self.log(Item::Operation(operation));
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
