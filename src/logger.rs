use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
};

use crate::{
    action::Action,
    card::{Card, Color, Splay},
    card_pile::CardOrder,
    error::InnResult,
    game::{PlayerId, Players},
    observation::SingleAchievementView,
    structure::Place,
};

#[derive(Clone, Debug)]
pub enum SimpleOp {
    Draw,
    Meld,
    Tuck,
    Score,
    Return,
    DrawAndMeld,
    DrawAndScore,
    DrawAndTuck,
}

#[derive(Clone, Debug)]
pub enum Operation<'c> {
    Splay(PlayerId, Color, Splay),
    Transfer(Place, Place, &'c Card),
    Exchange(Place, Place, Vec<&'c Card>, Vec<&'c Card>),
    SimpleOp(SimpleOp, PlayerId, &'c Card, Place),
    Achieve(PlayerId, SingleAchievementView),
}

// TODO: GameStart, GameEnd message, etc.
#[derive(Clone, Debug)]
pub enum Item<'c> {
    Action(Action),
    Operation(Operation<'c>),
    NextAction(PlayerId),
    ChangeTurn(PlayerId, PlayerId), // last player, next player
}

#[derive(Default)]
pub struct Subject<'c> {
    // There's already a RefCell outside the Vec, because need to filter out
    // empty observers in notify(). But, inside it still need to be Weak<RefCell>,
    // because we want to make it general so that the observers can borrow
    // each other in its turn. (Although not a common case)
    observers: RefCell<Vec<Weak<RefCell<dyn InternalObserver<'c>>>>>,
    owned_observers: Vec<RefCell<Box<dyn InternalObserver<'c> + 'c>>>,
    ext_observers: RefCell<Vec<Weak<RefCell<dyn Observer<'c>>>>>,
    owned_ext_observers: Vec<RefCell<Box<dyn Observer<'c> + 'c>>>,
    waiting: RefCell<VecDeque<Item<'c>>>,
    processing: RefCell<()>, // can't be bool because we need to mutate it
}

impl<'c> Subject<'c> {
    pub fn new() -> Self {
        Self {
            observers: RefCell::new(Vec::new()),
            owned_observers: Vec::new(),
            ext_observers: RefCell::new(Vec::new()),
            owned_ext_observers: Vec::new(),
            waiting: RefCell::new(VecDeque::new()),
            processing: RefCell::new(()),
        }
    }

    /// Register an external observer to the system.
    ///
    /// The caller should have a strong reference of the observer to prevent dropping.
    pub fn register_external(&mut self, new_observer: &Rc<RefCell<dyn Observer<'c>>>) {
        self.ext_observers
            .borrow_mut()
            .push(Rc::downgrade(new_observer));
    }

    /// Register a permanent external observer to the system.
    pub fn register_external_owned(&mut self, new_observer: impl Observer<'c> + 'c) {
        self.owned_ext_observers
            .push(RefCell::new(Box::new(new_observer)));
    }

    /// Register an internal observer to the system.
    ///
    /// The caller should have a strong reference of the observer to prevent dropping.
    pub fn register_internal(&mut self, new_observer: &Rc<RefCell<dyn InternalObserver<'c>>>) {
        self.observers
            .borrow_mut()
            .push(Rc::downgrade(new_observer));
    }

    /// Register a permanent internal observer to the system.
    pub fn register_internal_owned(&mut self, new_observer: impl InternalObserver<'c> + 'c) {
        self.owned_observers
            .push(RefCell::new(Box::new(new_observer)));
    }

    // must be immutable &self, because there may be multiple calls in the stack
    pub fn notify(&self, item: Item<'c>, game: &Players<'c>) -> InnResult<()> {
        self.waiting.borrow_mut().push_back(item);

        let check = self.processing.try_borrow_mut();

        if check.is_err() {
            return Ok(());
        }

        let processing = check.unwrap();

        // process until no message delay.
        loop {
            // using while let has lifetime issue
            let next = self.waiting.borrow_mut().pop_front();
            if next.is_none() {
                break;
            }

            let item = next.unwrap();

            // first notify external observers, which may log events and don't modify game state,
            // so we won't worry about multiple RefCell borrow_mut
            for owned_observer in self.owned_ext_observers.iter() {
                owned_observer.borrow_mut().on_notify(&item);
            }
            self.ext_observers.borrow_mut().retain_mut(|observer| {
                if let Some(active_observer) = observer.upgrade() {
                    active_observer.borrow_mut().on_notify(&item);
                    true
                } else {
                    false
                }
            });

            // second notify internal observers, letting them modify the game state and send new events
            for owned_observer in self.owned_observers.iter() {
                owned_observer.borrow_mut().update(&item, game)?;
            }
            // can't retain_mut directly because of InnResult
            // i.e., the list must be filtered after update and possibly return earlier.
            for observer in self.observers.borrow().iter() {
                if let Some(active_observer) = observer.upgrade() {
                    active_observer.borrow_mut().update(&item, game)?;
                }
            }
            self.observers
                .borrow_mut()
                .retain_mut(|o| o.upgrade().is_some());
        }

        // drop explicitly, because I don't know if it'll be optimized to be dropped earlier
        drop(processing);

        Ok(())
    }

    pub fn act(&self, action: Action, game: &Players<'c>) -> InnResult<()> {
        self.notify(Item::Action(action), game)
    }

    pub fn operate(&self, operation: Operation<'c>, game: &Players<'c>) -> InnResult<()> {
        self.notify(Item::Operation(operation), game)
    }
}

pub trait Observer<'c> {
    // doesn't modify game state
    // does it need InnResult?
    fn on_notify(&mut self, event: &Item<'c>);
}

pub trait InternalObserver<'c> {
    fn update(&mut self, event: &Item<'c>, game: &Players<'c>) -> InnResult<()>;
}

// there's really no way to factor the type
#[allow(clippy::type_complexity)]
pub struct FnInternalObserver<'c>(Box<dyn FnMut(&Item<'c>, &Players<'c>) -> InnResult<()> + 'c>);

impl<'c> FnInternalObserver<'c> {
    pub fn new(f: impl FnMut(&Item<'c>, &Players<'c>) -> InnResult<()> + 'c) -> Self {
        Self(Box::new(f))
    }
}

impl<'c> InternalObserver<'c> for FnInternalObserver<'c> {
    fn update(&mut self, event: &Item<'c>, game: &Players<'c>) -> InnResult<()> {
        self.0(event, game)
    }
}

pub struct FnObserver<'c>(Box<dyn FnMut(&Item<'c>) + 'c>);

impl<'c> FnObserver<'c> {
    pub fn new(f: impl FnMut(&Item<'c>) + 'c) -> Self {
        Self(Box::new(f))
    }
}

impl<'c> Observer<'c> for FnObserver<'c> {
    fn on_notify(&mut self, event: &Item<'c>) {
        self.0(event);
    }
}

#[derive(Clone)]
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

    pub fn current_game(&self) -> Option<&Game<'c>> {
        self.current_game.as_ref()
    }
}

impl<'c> Observer<'c> for Logger<'c> {
    fn on_notify(&mut self, event: &Item<'c>) {
        self.log(event.clone());
    }
}
