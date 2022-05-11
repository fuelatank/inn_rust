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
    }
}

pub enum Yield<'a> {
    ChooseAnyCard {
        min_num: u8,
        max_num: Option<u8>,
        from: Vec<&'a Card>,
    },
    ChooseAnOpponent,
}

pub type ShareFlow = Box<dyn for<'a> Fn(&'a Player<'a>, &'a Game<'a>) -> ExecutingState<'a>>;
pub type DemandFlow =
    Box<dyn for<'a> Fn(&'a Player<'a>, &'a Player<'a>, &'a Game<'a>) -> ExecutingState<'a>>;

mod tests {
    //use crate::game::transfer_elem;
    use super::*;
    use crate::enums::Splay;
    use crate::card::Achievement;
    use crate::containers::Addable;
    use crate::containers::VecSet;
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
            return ExecutingState::ChooseAnyCard {
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
                    return ExecutingState::Done;
                }),
            };
        })
    }

    fn _opticsxx() -> ShareFlow {
        Box::new(|player, _game| {
            let card = player.draw_and_meld(3).unwrap();
            if card.contains(Icon::Crown) {
                player.draw_and_score(4);
                return ExecutingState::Done;
            } else {
                return ExecutingState::ChooseAnyCard {
                    min_num: 1,
                    max_num: Some(1),
                    from: player.score_pile.borrow().as_vec(),
                    callback: Box::new(move |cards: Vec<&Card>| {
                        match cards.get(0) {
                            Some(&card) => ExecutingState::ChooseAnOpponent {
                                callback: Box::new(move |opponent: &Player| {
                                    transfer(
                                        &player.score_pile,
                                        &opponent.score_pile,
                                        card,
                                    );
                                    ExecutingState::Done
                                }),
                            },
                            None => ExecutingState::Done
                        }
                    }),
                };
            }
        })
    }

    fn _code_of_laws() -> ShareFlow {
        Box::new(|player, _game| {
            return ExecutingState::ChooseAnyCard {
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
                    match cards.get(0) {
                        Some(&card) => {
                            player.tuck(card);
                            if !player.is_splayed(card.color(), Splay::Left) {
                                ExecutingState::ChooseYn {
                                    callback: Box::new(move |yn| {
                                        if yn {
                                            player.splay(card.color(), Splay::Left);
                                        }
                                        ExecutingState::Done
                                    })
                                }
                            } else {
                                ExecutingState::Done
                            }
                        },
                        None => ExecutingState::Done,
                    }
                })
            };
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
