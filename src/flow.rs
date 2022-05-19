use crate::action::ExecutingAction;
use crate::card::Card;
use crate::containers::CardSet;
use crate::enums::Icon;
use crate::game::Game;
use crate::player::Player;
use crate::state::{Choose, ExecutionState};
use generator::{done, Gn, LocalGenerator, Scope};

pub type FlowState<'c, 'g> = LocalGenerator<'g, ExecutingAction<'c, 'g>, ExecutionState<'c, 'g>>;

pub type ShareFlow = for<'c, 'g> fn(&'g Player<'c>, &'g Game<'c>) -> FlowState<'c, 'g>;
pub type DemandFlow =
    for<'c, 'g> fn(&'g Player<'c>, &'g Player<'c>, &'g Game<'c>) -> FlowState<'c, 'g>;

mod tests {
    //use crate::game::transfer_elem;
    use super::*;
    use crate::card::Achievement;
    use crate::containers::{transfer, Addable, VecSet};
    use crate::enums::Splay;

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
        Gn::new_scoped_local(move |mut s: Scope<ExecutingAction, _>| {
            opponent.draw(1);
            let age = opponent.age();
            let cards = s
                .yield_(ExecutionState::new(
                    opponent,
                    Choose::Card {
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
                ))
                .expect("Generator got None")
                .cards();
            transfer(&opponent.hand, &player.hand, cards[0]);
            generator::done!()
        })
    }

    fn _opticsxx<'c, 'g>(player: &'g Player<'c>, _game: &'g Game<'c>) -> FlowState<'c, 'g> {
        Gn::new_scoped_local(move |mut s: Scope<ExecutingAction, _>| {
            let card = player.draw_and_meld(3).unwrap();
            if card.contains(Icon::Crown) {
                player.draw_and_score(4);
                done!()
            } else {
                let opt_card = s
                    .yield_(ExecutionState::new(
                        player,
                        Choose::Card {
                            min_num: 1,
                            max_num: Some(1),
                            from: player.score_pile.borrow().as_vec(),
                        },
                    ))
                    .expect("Generator got None")
                    .card();
                let card = match opt_card {
                    Some(c) => c,
                    None => done!(),
                };
                let opponent = s
                    .yield_(ExecutionState::new(player, Choose::Opponent))
                    .expect("Generator got None")
                    .player();
                transfer(&player.score_pile, &opponent.score_pile, card);
                done!()
            }
        })
    }

    fn _code_of_laws<'c, 'g>(player: &'g Player<'c>, _game: &'g Game<'c>) -> FlowState<'c, 'g> {
        Gn::new_scoped_local(move |mut s: Scope<ExecutingAction, _>| {
            let opt_card = s
                .yield_(ExecutionState::new(
                    player,
                    Choose::Card {
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
                ))
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
            if s.yield_(ExecutionState::new(player, Choose::Yn))
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
        let mut game: Game = Game::new();
        game.add_player(
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
        );
        //let the_wheel = vec![];
        //let chemistry1 = vec![];
        //let optics = vec![];
    }
}
