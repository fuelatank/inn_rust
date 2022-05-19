use crate::action::{Action, StepAction};
use crate::card_pile::MainCardPile;
use crate::containers::{BoxAchievementSet, BoxCardSet};
use crate::player::Player;
use crate::state::State;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub type RcCell<T> = Rc<RefCell<T>>;

pub struct Game<'c> {
    main_card_pile: RcCell<MainCardPile<'c>>,
    players: Vec<Player<'c>>,
}

impl<'c> Game<'c> {
    pub fn new() -> Game<'c> {
        Game {
            main_card_pile: Rc::new(RefCell::new(MainCardPile::new())),
            players: vec![],
        }
    }

    pub fn add_player(
        &mut self,
        hand: BoxCardSet<'c>,
        score_pile: BoxCardSet<'c>,
        achievements: BoxAchievementSet<'c>,
    ) {
        let id = self.players.len();
        self.players.push(Player::new(
            id,
            Rc::clone(&self.main_card_pile),
            hand,
            score_pile,
            achievements,
        ))
    }

    pub fn players(&self) -> &Vec<Player<'c>> {
        &self.players
    }

    pub fn players_from(&self, main_player_id: usize) -> impl Iterator<Item = &Player<'c>> {
        (0..self.players.len())
            .map(move |i| &self.players[(i + main_player_id) % self.players.len()])
    }
}

struct OuterGame<'c, 'g> {
    players: Game<'c>,
    current_player_id: usize,
    is_second_action: bool,
    state: Cell<State<'c, 'g>>,
}

impl<'c, 'g> OuterGame<'c, 'g> {
    fn is_available_step_action(&self, action: &StepAction<'c>) -> bool {
        match action {
            StepAction::Draw => true,
            StepAction::Meld(c) => {
                let player = &self.players.players[self.current_player_id];
                player.hand.borrow().as_vec().contains(c)
            }
            StepAction::Achieve(_) => todo!(),
            StepAction::Execute(c) => {
                let player = &self.players.players[self.current_player_id];
                player.board().borrow().contains(c)
            }
        }
    }

    pub fn step(&'g self, action: Action<'c, '_>) {
        let next_state = match (self.state.take(), action) {
            (State::Main, Action::Step(action)) => {
                let player = &self.players.players[self.current_player_id];
                match action {
                    StepAction::Draw => {
                        player.draw(player.age());
                        todo!()
                    }
                    StepAction::Meld(card) => {
                        player.meld(card);
                        todo!()
                    }
                    StepAction::Achieve(age) => {
                        todo!()
                    }
                    StepAction::Execute(card) => {
                        State::Executing(player.execute(card, &self.players))
                    }
                }
            }
            (State::Executing(state), Action::Executing(action)) => todo!(),
            _ => panic!("State and action mismatched"),
        };
        self.state.set(next_state);
    }
}

mod tests {
    use super::*;
    use crate::containers::VecSet;

    #[test]
    fn create_game_player() {
        let mut game = Game::new();
        game.add_player(
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
        );
        game.add_player(
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
        );
    }
}
