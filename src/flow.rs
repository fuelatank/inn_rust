use crate::card::Card;
use crate::containers::CardSet;
use crate::enums::Icon;
use crate::game::Game;
use crate::game::Player;
use generator::{done, Gn, LocalGenerator, Scope};

pub enum ExecutingState<'a> {
    Done,
    ChooseAnyCard {
        min_num: u8,
        max_num: Option<u8>,
        from: Vec<&'a Card>,
        callback: Box<dyn FnOnce(Vec<&'a Card>) -> ExecutingState<'a> + 'a>,
    },
    ChooseAnOpponent {
        callback: Box<dyn FnOnce(&'a Player<'a>) -> ExecutingState<'a> + 'a>,
    },
    ChooseYn {
        callback: Box<dyn FnOnce(bool) -> ExecutingState<'a> + 'a>,
    },
}

pub enum ExecutionState<'a> {
    Card {
        min_num: u8,
        max_num: Option<u8>,
        from: Vec<&'a Card>,
        callback: Box<dyn FnOnce(Vec<&'a Card>) -> Option<ExecutionState<'a>> + 'a>,
    },
    Opponent {
        callback: Box<dyn FnOnce(&'a Player<'a>) -> Option<ExecutionState<'a>> + 'a>,
    },
    Yn {
        callback: Box<dyn FnOnce(bool) -> Option<ExecutionState<'a>> + 'a>,
    },
}

pub enum State<'a> {
    Card {
        min_num: u8,
        max_num: Option<u8>,
        from: Vec<&'a Card>,
    },
    Opponent,
    Yn,
}

pub enum Action<'a> {
    Card(Vec<&'a Card>),
    Opponent(&'a Player<'a>),
    Yn(bool),
}

impl<'a> Action<'a> {
    fn card(self) -> Option<&'a Card> {
        if let Action::Card(cards) = self {
            if cards.len() == 0 {
                None
            } else if cards.len() == 1 {
                Some(cards[0])
            } else {
                panic!("Error when unwrapping Action to one card")
            }
        } else {
            panic!("Error when unwrapping Action to one card")
        }
    }
    fn cards(self) -> Vec<&'a Card> {
        if let Action::Card(cards) = self {
            cards
        } else {
            panic!("Error when unwrapping Action to cards")
        }
    }
    fn player(self) -> &'a Player<'a> {
        if let Action::Opponent(player) = self {
            player
        } else {
            panic!("Error when unwrapping Action to player")
        }
    }
    fn yn(self) -> bool {
        if let Action::Yn(yn) = self {
            yn
        } else {
            panic!("Error when unwrapping Action to yn")
        }
    }
}

pub type ShareFlow =
    Box<dyn for<'a> Fn(&'a Player<'a>, &'a Game<'a>) -> Option<ExecutionState<'a>>>;
pub type DemandFlow =
    Box<dyn for<'a> Fn(&'a Player<'a>, &'a Player<'a>, &'a Game<'a>) -> Option<ExecutionState<'a>>>;
pub type FlowState<'a> = LocalGenerator<'a, Action<'a>, State<'a>>;

struct ESC<'a> {
    state: Option<ExecutionState<'a>>,
}

impl<'a> ESC<'a> {
    fn next(&mut self, action: Action<'a>) -> Option<State> {
        let state = self.state.take().expect("already finished");
        let (inner, outer) = match (state, action) {
            (
                ExecutionState::Card {
                    min_num,
                    max_num,
                    from,
                    callback,
                },
                Action::Card(action),
            ) => (
                callback(action),
                State::Card {
                    min_num,
                    max_num,
                    from,
                },
            ),
            (ExecutionState::Opponent { callback }, Action::Opponent(opponent)) => {
                (callback(opponent), State::Opponent)
            }
            (ExecutionState::Yn { callback }, Action::Yn(yn)) => (callback(yn), State::Yn),
            _ => unreachable!(),
        };
        self.state = inner;
        self.state.as_ref().map(|_| outer)
    }
}

mod tests {
    //use crate::game::transfer_elem;
    use super::*;
    use crate::card::Achievement;
    use crate::containers::Addable;
    use crate::containers::VecSet;
    use crate::enums::Splay;
    use crate::game::transfer;
    use crate::game::Player;

    fn _chemistry2<'a, T: CardSet<'a, Card>, U: Addable<'a, Achievement> + Default>(
    ) -> Box<dyn Fn(&mut Game, usize)> {
        // Player is inside Game
        // One player must be placed inside one game
        // Player is created when that Game is created
        Box::new(|_game, _player| {
            //game.draw_and_score(player, player.age() + 1);
        })
    }

    fn _archery<'a>(
        player: &'a Player<'a>,
        opponent: &'a Player<'a>,
        _game: &'a Game<'a>,
    ) -> FlowState<'a> {
        Gn::new_scoped_local(move |mut s: Scope<Action, _>| {
            opponent.draw(1);
            let age = opponent.age();
            let cards = s
                .yield_(State::Card {
                    min_num: 1,
                    max_num: Some(1),
                    from: opponent
                        .hand
                        .borrow()
                        .as_vec()
                        .into_iter()
                        .filter(|c| c.age() == age)
                        .collect(),
                })
                .expect("Generator got None")
                .cards();
            transfer(&opponent.hand, &player.hand, cards[0]);
            generator::done!()
        })
    }

    fn _opticsxx<'a>(player: &'a Player<'a>, _game: &'a Game<'a>) -> FlowState<'a> {
        Gn::new_scoped_local(move |mut s: Scope<Action, _>| {
            let card = player.draw_and_meld(3).unwrap();
            if card.contains(Icon::Crown) {
                player.draw_and_score(4);
                done!()
            } else {
                let opt_card = s
                    .yield_(State::Card {
                        min_num: 1,
                        max_num: Some(1),
                        from: player.score_pile.borrow().as_vec(),
                    })
                    .expect("Generator got None")
                    .card();
                let card = match opt_card {
                    Some(c) => c,
                    None => done!(),
                };
                let opponent = s
                    .yield_(State::Opponent)
                    .expect("Generator got None")
                    .player();
                transfer(&player.score_pile, &opponent.score_pile, card);
                done!()
            }
        })
    }

    fn _code_of_laws<'a>(player: &'a Player<'a>, _game: &'a Game<'a>) -> FlowState<'a> {
        Gn::new_scoped_local(move |mut s: Scope<Action, _>| {
            let opt_card = s
                .yield_(State::Card {
                    min_num: 0,
                    max_num: Some(1),
                    from: player
                        .hand
                        .borrow()
                        .as_vec()
                        .into_iter()
                        .filter(|card| !player.board().borrow().get_stack(card.color()).is_empty())
                        .collect(),
                })
                .expect("Generator got None")
                .card();
            let card = match opt_card {
                Some(c) => c,
                None => done!(),
            };
            player.tuck(card);
            if player.is_splayed(card.color(), Splay::Left) {
                done!()
            }
            if s.yield_(State::Yn).expect("Generator got None").yn() {
                player.splay(card.color(), Splay::Left);
            }
            done!()
        })
    }

    #[test]
    fn name() {
        let game: Game = Game::new();
        game.add_player(
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
        );
        let the_wheel = vec![];
        let chemistry1 = vec![];
        let optics = vec![];
    }
}
