/*
trait GameState {}
trait GameAction {}

trait StepGame {
    type State: GameState;
    type Action: GameAction;
    fn reset(&self);
    fn get_state(&self) -> Self::State;
    fn step(&self, action: Self::Action);
}

struct Innovation {}

impl Innovation {
    fn new() -> Innovation {}
    fn set_cards(&self, cards: Vec<Card>) {
        self.cards = cards;
        self.reset();
    }
    fn reset(&self) {
        self.main_card_pile.clear();
        self.players.iter_mut().map(|p| p.clear());
        self.main_card_pile.add_cards(&self.cards);
        self.initiate()
    }
    fn initiate(&self) {
        for p in self.players.iter_mut() {
            p.draw(1);
            p.draw(1);
        }
    }
    fn step(&self, action: Action) {
        match self.get_state() {
            State::Beginning() => {
            }
        }
    }
}*/