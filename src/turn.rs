use serde::Serialize;

use crate::{error::InnResult, game::Players, logger::Item};

#[derive(Debug, Serialize)]
pub struct Turn {
    step: usize,
    num_players: usize,
    current_player: usize,
    is_second_step: bool,
}

impl Turn {
    pub fn new(num_players: usize) -> Turn {
        Turn {
            step: 0,
            num_players,
            current_player: 0,
            is_second_step: true,
        }
    }

    pub fn current_player(&self) -> usize {
        self.current_player
    }

    pub fn is_second_step(&self) -> bool {
        self.is_second_step
    }

    pub fn player_id(&self) -> usize {
        self.current_player
    }

    pub fn next_step(&mut self) {
        self.step += 1;
        if self.is_second_step {
            self.current_player = (self.current_player + 1) % self.num_players;
        }
        self.is_second_step = !self.is_second_step;
    }
}

pub struct TurnBuilder {
    first_player: usize,
    is_second_step: bool,
}

impl TurnBuilder {
    pub fn new() -> TurnBuilder {
        TurnBuilder {
            first_player: 0,
            is_second_step: true,
        }
    }

    pub fn first_player(mut self, player: usize) -> TurnBuilder {
        self.first_player = player;
        self
    }

    pub fn second_step(mut self, is_second_step: bool) -> TurnBuilder {
        self.is_second_step = is_second_step;
        self
    }

    pub fn build(self, num_players: usize) -> Turn {
        Turn {
            step: 0,
            num_players,
            current_player: self.first_player % num_players,
            is_second_step: self.is_second_step,
        }
    }
}

impl Default for TurnBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LoggingTurn<'c, 'g> {
    turn: Turn,
    game: &'g Players<'c>,
}

impl<'c, 'g> LoggingTurn<'c, 'g> {
    pub fn new(turn: Turn, game: &'g Players<'c>) -> Self {
        Self { turn, game }
    }

    pub fn current_player(&self) -> usize {
        self.turn.current_player()
    }

    pub fn is_second_step(&self) -> bool {
        self.turn.is_second_step()
    }

    pub fn player_id(&self) -> usize {
        self.turn.player_id()
    }

    pub fn next_step(&mut self) -> InnResult<()> {
        let original_player = self.turn.player_id();
        self.turn.next_step();
        let current_player = self.turn.player_id();
        if original_player == current_player {
            self.game.notify(Item::NextAction(current_player))
        } else {
            self.game
                .notify(Item::ChangeTurn(original_player, current_player))
        }
    }

    pub fn turn(&self) -> &Turn {
        &self.turn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn() {
        let mut t1 = TurnBuilder::new().first_player(1).build(5);
        assert_eq!(t1.player_id(), 1);
        assert_eq!(t1.is_second_step(), true);
        t1.next_step();
        assert_eq!(t1.player_id(), 2);
        assert_eq!(t1.is_second_step(), false);
        t1.next_step();
        assert_eq!(t1.player_id(), 2);
        assert_eq!(t1.is_second_step(), true);
        t1.next_step();
        assert_eq!(t1.player_id(), 3);
        assert_eq!(t1.is_second_step(), false);
        t1.next_step();
        assert_eq!(t1.player_id(), 3);
        assert_eq!(t1.is_second_step(), true);
        t1.next_step();
        assert_eq!(t1.player_id(), 4);
        assert_eq!(t1.is_second_step(), false);
        t1.next_step();
        assert_eq!(t1.player_id(), 4);
        assert_eq!(t1.is_second_step(), true);
        t1.next_step();
        assert_eq!(t1.player_id(), 0);
        assert_eq!(t1.is_second_step(), false);
        t1.next_step();
        assert_eq!(t1.player_id(), 0);
        assert_eq!(t1.is_second_step(), true);
        t1.next_step();
        assert_eq!(t1.player_id(), 1);
        assert_eq!(t1.is_second_step(), false);
        t1.next_step();
        assert_eq!(t1.player_id(), 1);
        assert_eq!(t1.is_second_step(), true);
        t1.next_step();
        t1.next_step();
        t1.next_step();
        t1.next_step();
        assert_eq!(t1.player_id(), 3);
        assert_eq!(t1.is_second_step(), true);
    }
}
