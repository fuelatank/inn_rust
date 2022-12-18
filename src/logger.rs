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
pub enum SimpleOp {
    Draw,
    Meld,
    Tuck,
    Score,
    Return,
}

#[derive(Clone)]
pub enum Operation<'c> {
    Splay(PlayerId, Color, Splay),
    Transfer(Place, Place, &'c Card),
    SimpleOp(SimpleOp, PlayerId, &'c Card),
}

#[derive(Clone)]
pub enum Item<'c> {
    Action(Action),
    Operation(Operation<'c>),
    NextAction(PlayerId),
    ChangeTurn(PlayerId, PlayerId), // last player, next player
}

pub struct Subject<'c> {
    observers: Vec<Weak<RefCell<dyn InternalObserver<'c>>>>,
    owned_observers: Vec<Box<dyn InternalObserver<'c> + 'c>>,
    ext_observers: Vec<Weak<RefCell<dyn Observer<'c>>>>,
    owned_ext_observers: Vec<Box<dyn Observer<'c> + 'c>>,
}

impl<'c> Subject<'c> {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
            owned_observers: Vec::new(),
            ext_observers: Vec::new(),
            owned_ext_observers: Vec::new(),
        }
    }

    /// Register an external observer to the system.
    ///
    /// The caller should have a strong reference of the observer to prevent dropping.
    pub fn register_external(&mut self, new_observer: &Rc<RefCell<dyn Observer<'c>>>) {
        self.ext_observers.push(Rc::downgrade(new_observer));
    }

    /// Register a permanent external observer to the system.
    pub fn register_external_owned(&mut self, new_observer: impl Observer<'c> + 'c) {
        self.owned_ext_observers.push(Box::new(new_observer));
    }

    /// Register an internal observer to the system.
    ///
    /// The caller should have a strong reference of the observer to prevent dropping.
    pub fn register_internal(&mut self, new_observer: &Rc<RefCell<dyn InternalObserver<'c>>>) {
        self.observers.push(Rc::downgrade(new_observer));
    }

    /// Register a permanent internal observer to the system.
    pub fn register_internal_owned(&mut self, new_observer: impl InternalObserver<'c> + 'c) {
        self.owned_observers.push(Box::new(new_observer));
    }

    pub fn notify(&mut self, item: Item<'c>, game: &Players<'c>) {
        // first notify external observers, who often log events and don't modify game state
        for owned_observer in self.owned_ext_observers.iter_mut() {
            owned_observer.on_notify(&item);
        }
        self.ext_observers.retain_mut(|observer| {
            observer
                .upgrade()
                .map(|active_observer| active_observer.borrow_mut().on_notify(&item))
                .is_some()
        });

        // second notify internal observers, who may modify game state and send new events
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

pub trait InternalObserver<'c> {
    fn on_notify(&mut self, event: &Item<'c>, game: &Players<'c>);
}

pub trait Observer<'c> {
    // doesn't modify game state
    fn on_notify(&mut self, event: &Item<'c>);
}

impl<'c, T: Observer<'c>> InternalObserver<'c> for T {
    fn on_notify(&mut self, event: &Item<'c>, _game: &Players<'c>) {
        self.on_notify(event);
    }
}

pub struct FnObserver<'c>(Box<dyn FnMut(&Item<'c>, &Players<'c>) + 'c>);

impl<'c> FnObserver<'c> {
    pub fn new(f: impl FnMut(&Item<'c>, &Players<'c>) + 'c) -> Self {
        Self(Box::new(f))
    }
}

impl<'c> InternalObserver<'c> for FnObserver<'c> {
    fn on_notify(&mut self, event: &Item<'c>, game: &Players<'c>) {
        self.0(event, game);
    }
}

pub struct FnPureObserver<'c>(Box<dyn FnMut(&Item<'c>) + 'c>);

impl<'c> FnPureObserver<'c> {
    pub fn new(f: impl FnMut(&Item<'c>) + 'c) -> Self {
        Self(Box::new(f))
    }
}

impl<'c> Observer<'c> for FnPureObserver<'c> {
    fn on_notify(&mut self, event: &Item<'c>) {
        self.0(event);
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

impl<'c> Observer<'c> for Logger<'c> {
    fn on_notify(&mut self, event: &Item<'c>) {
        self.log(event.clone());
    }
}
