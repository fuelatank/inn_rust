use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{
    action::Action,
    card::Card,
    card_pile::CardOrder,
    enums::{Color, Splay},
    game::{PlayerId, Players},
    structure::Place,
};

#[derive(Clone)]
pub enum Operation<'c> {
    Splay(PlayerId, Color, Splay),
    Transfer(Place, Place, &'c Card),
}

#[derive(Clone)]
pub enum Item<'c> {
    Action(Action),
    Operation(Operation<'c>),
}

pub struct Subject<'c> {
    observers: Vec<Weak<RefCell<dyn Observer<'c>>>>,
    owned_observers: Vec<Box<dyn Observer<'c>>>,
}

impl<'c> Subject<'c> {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
            owned_observers: Vec::new(),
        }
    }

    /// Register an observer to the system.
    ///
    /// The caller should have a strong reference of the observer to prevent dropping.
    pub fn register(&mut self, new_observer: &Rc<RefCell<dyn Observer<'c>>>) {
        self.observers.push(Rc::downgrade(new_observer));
    }

    /// Register a permanent observer to the system.
    pub fn register_owned(&mut self, new_observer: impl Observer<'c> + 'static) {
        self.owned_observers.push(Box::new(new_observer));
    }

    pub fn notify(&mut self, item: Item<'c>, game: &Players<'c>) {
        for owned_observer in self.owned_observers.iter_mut() {
            owned_observer.on_notify(&item, game);
        }
        self.observers.retain_mut(|observer| {
            observer
                .upgrade()
                .map(|active_observer| active_observer.borrow_mut().on_notify(&item, game))
                .is_some()
        });
    }

    pub fn act(&mut self, action: Action, game: &Players<'c>) {
        self.notify(Item::Action(action), game);
    }

    pub fn operate(&mut self, operation: Operation<'c>, game: &Players<'c>) {
        self.notify(Item::Operation(operation), game);
    }
}

pub trait Observer<'c> {
    fn on_notify(&mut self, event: &Item<'c>, game: &Players<'c>);
}

struct FnObserver<'c>(Box<dyn FnMut(&Item<'c>, &Players<'c>)>);

impl<'c> Observer<'c> for FnObserver<'c> {
    fn on_notify(&mut self, event: &Item<'c>, game: &Players<'c>) {
        self.0(event, game);
    }
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
