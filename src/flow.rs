use crate::card::Card;
use crate::containers::CardSet;
use crate::enums::Icon;
use crate::game::Game;
use crate::game::Player;

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

pub enum Yield<'a> {
    ChooseAnyCard {
        min_num: u8,
        max_num: Option<u8>,
        from: Vec<&'a Card>,
    },
    ChooseAnOpponent,
}

pub type ShareFlow =
    Box<dyn for<'a> Fn(&'a Player<'a>, &'a Game<'a>) -> Option<ExecutionState<'a>>>;
pub type DemandFlow =
    Box<dyn for<'a> Fn(&'a Player<'a>, &'a Player<'a>, &'a Game<'a>) -> Option<ExecutionState<'a>>>;

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
            ) => (callback(action), State::Card { min_num, max_num, from }),
            (ExecutionState::Opponent { callback }, Action::Opponent(opponent)) => {
                (callback(opponent), State::Opponent)
            }
            (ExecutionState::Yn { callback }, Action::Yn(yn)) => (callback(yn),State::Yn),
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

    fn _archery() -> DemandFlow {
        Box::new(|player, opponent, _game| {
            opponent.draw(1);
            let age = opponent.age();
            Some(ExecutionState::Card {
                min_num: 1,
                max_num: Some(1),
                from: opponent
                    .hand
                    .borrow()
                    .as_vec()
                    .into_iter()
                    .filter(|c| c.age() == age)
                    .collect(),
                callback: Box::new(move |cards: Vec<&Card>| {
                    transfer(&opponent.hand, &player.hand, cards[0]);
                    None
                }),
            })
        })
    }

    fn _opticsxx() -> ShareFlow {
        Box::new(|player, _game| {
            let card = player.draw_and_meld(3).unwrap();
            if card.contains(Icon::Crown) {
                player.draw_and_score(4);
                return None;
            } else {
                return Some(ExecutionState::Card {
                    min_num: 1,
                    max_num: Some(1),
                    from: player.score_pile.borrow().as_vec(),
                    callback: Box::new(move |cards: Vec<&Card>| {
                        let card = *cards.get(0)?;
                        Some(ExecutionState::Opponent {
                            callback: Box::new(move |opponent: &Player| {
                                transfer(&player.score_pile, &opponent.score_pile, card);
                                None
                            }),
                        })
                    }),
                });
            }
        })
    }

    fn _code_of_laws() -> ShareFlow {
        Box::new(|player, _game| {
            return Some(ExecutionState::Card {
                min_num: 0,
                max_num: Some(1),
                from: player
                    .hand
                    .borrow()
                    .as_vec()
                    .into_iter()
                    .filter(|card| !player.board().borrow().get_stack(card.color()).is_empty())
                    .collect(),
                callback: Box::new(move |cards| {
                    let card = *cards.get(0)?;
                    player.tuck(card);
                    if player.is_splayed(card.color(), Splay::Left) {
                        return None;
                    }
                    Some(ExecutionState::Yn {
                        callback: Box::new(move |yn| {
                            if yn {
                                player.splay(card.color(), Splay::Left);
                            }
                            None
                        }),
                    })
                }),
            });
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
