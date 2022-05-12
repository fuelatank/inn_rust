use crate::card::Card;
use crate::containers::CardSet;
use crate::enums::Icon;
use crate::game::Game;
use crate::game::Player;
use generator::{done, Gn, LocalGenerator, Scope};

pub enum Choose<'a> {
    Card {
        min_num: u8,
        max_num: Option<u8>,
        from: Vec<&'a Card>,
    },
    Opponent,
    Yn,
}

pub struct State<'c, 'g> {
    state: Choose<'c>,
    actor: &'g Player<'c>,
}

pub enum Action<'c, 'g> {
    Card(Vec<&'c Card>),
    Opponent(&'g Player<'c>),
    Yn(bool),
}

impl<'c, 'g> Action<'c, 'g> {
    fn card(self) -> Option<&'c Card> {
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
    fn cards(self) -> Vec<&'c Card> {
        if let Action::Card(cards) = self {
            cards
        } else {
            panic!("Error when unwrapping Action to cards")
        }
    }
    fn player(self) -> &'g Player<'c> {
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

pub type ShareFlow = for<'c, 'g> fn(&'g Player<'c>, &'g Game<'c>) -> FlowState<'c, 'g>;
pub type DemandFlow =
    for<'c, 'g> fn(&'g Player<'c>, &'g Player<'c>, &'g Game<'c>) -> FlowState<'c, 'g>;
pub type FlowState<'c, 'g> = LocalGenerator<'g, Action<'c, 'g>, State<'c, 'g>>;

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

    fn _archery<'c, 'g>(
        player: &'g Player<'c>,
        opponent: &'g Player<'c>,
        _game: &'g Game<'c>,
    ) -> FlowState<'c, 'g> {
        Gn::new_scoped_local(move |mut s: Scope<Action, _>| {
            opponent.draw(1);
            let age = opponent.age();
            let cards = s
                .yield_(State {
                    state: Choose::Card {
                        min_num: 1,
                        max_num: Some(1),
                        from: opponent
                            .hand
                            .borrow()
                            .as_vec()
                            .into_iter()
                            .filter(|c| c.age() == age)
                            .collect(),
                    },
                    actor: opponent,
                })
                .expect("Generator got None")
                .cards();
            transfer(&opponent.hand, &player.hand, cards[0]);
            generator::done!()
        })
    }

    fn _opticsxx<'c, 'g>(player: &'g Player<'c>, _game: &'g Game<'c>) -> FlowState<'c, 'g> {
        Gn::new_scoped_local(move |mut s: Scope<Action, _>| {
            let card = player.draw_and_meld(3).unwrap();
            if card.contains(Icon::Crown) {
                player.draw_and_score(4);
                done!()
            } else {
                let opt_card = s
                    .yield_(State {
                        state: Choose::Card {
                            min_num: 1,
                            max_num: Some(1),
                            from: player.score_pile.borrow().as_vec(),
                        },
                        actor: player,
                    })
                    .expect("Generator got None")
                    .card();
                let card = match opt_card {
                    Some(c) => c,
                    None => done!(),
                };
                let opponent = s
                    .yield_(State {
                        state: Choose::Opponent,
                        actor: player,
                    })
                    .expect("Generator got None")
                    .player();
                transfer(&player.score_pile, &opponent.score_pile, card);
                done!()
            }
        })
    }

    fn _code_of_laws<'c, 'g>(player: &'g Player<'c>, _game: &'g Game<'c>) -> FlowState<'c, 'g> {
        Gn::new_scoped_local(move |mut s: Scope<Action, _>| {
            let opt_card = s
                .yield_(State {
                    state: Choose::Card {
                        min_num: 0,
                        max_num: Some(1),
                        from: player
                            .hand
                            .borrow()
                            .as_vec()
                            .into_iter()
                            .filter(|card| {
                                !player.board().borrow().get_stack(card.color()).is_empty()
                            })
                            .collect(),
                    },
                    actor: player,
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
            if s.yield_(State {
                state: Choose::Yn,
                actor: player,
            })
            .expect("Generator got None")
            .yn()
            {
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
