use crate::flow::FlowState;
use crate::card::Card;
use crate::player::Player;

pub enum Choose<'a> {
    Card {
        min_num: u8,
        max_num: Option<u8>,
        from: Vec<&'a Card>,
    },
    Opponent,
    Yn,
}

pub struct ExecutionState<'c, 'g> {
    actor: &'g Player<'c>,
    state: Choose<'c>,
}

impl<'c, 'g> ExecutionState<'c, 'g> {
    pub fn new(actor: &'g Player<'c>, state: Choose<'c>) -> ExecutionState<'c, 'g> {
        ExecutionState { actor, state }
    }
}

// Main should include turn, num_actions, etc.
pub enum State<'c, 'g> {
    Main,
    Executing(FlowState<'c, 'g>),
}

impl<'c, 'g> Default for State<'c, 'g> {
    fn default() -> Self {
        State::Main
    }
}