use std::{
    cell::RefCell,
    rc::Rc,
};

use generator::{done, Gn, Scope};

use crate::{
    action::RefChoice,
    card::{Card, Dogma},
    enums::{Icon, Splay},
    flow::{DemandFlow, ShareFlow},
    game::{Players, RcCell},
    player::Player,
    state::{Choose, ExecutionState},
    structure::Place,
};

// which lifetime in Scope???
struct Context<'a, 'c, 'g> {
    s: Scope<'a, RefChoice<'c, 'g>, ExecutionState<'c, 'g>>,
}

impl<'a, 'c, 'g> Context<'a, 'c, 'g> {
    fn new(s: Scope<'a, RefChoice<'c, 'g>, ExecutionState<'c, 'g>>) -> Context<'a, 'c, 'g> {
        Context { s }
    }

    fn choose_one_card(&mut self, player: &'g Player<'c>, from: Vec<&'c Card>) -> Option<&'c Card> {
        let cards = self
            .s
            .yield_(ExecutionState::new(
                player,
                Choose::Card {
                    min_num: 1,
                    max_num: Some(1),
                    from,
                },
            ))
            .expect("Generator got None")
            .cards();
        debug_assert!(cards.len() <= 1);
        if !cards.is_empty() {
            Some(cards[0])
        } else {
            None
        }
    }

    fn choose_yn(&mut self, player: &'g Player<'c>) -> bool {
        self.s
            .yield_(ExecutionState::new(player, Choose::Yn))
            .expect("Generator got None")
            .yn()
    }

    fn may<T, F>(&mut self, player: &'g Player<'c>, f: F) -> Option<T>
    where
        F: FnOnce(&mut Context<'a, 'c, 'g>) -> T,
    {
        self.choose_yn(player).then(|| f(self))
    }
}

fn shared<F>(f: F) -> ShareFlow
where
    F: for<'a, 'c, 'g> Fn(&'g Player<'c>, &'g Players<'c>, Context<'a, 'c, 'g>) + 'static,
{
    // convert a ctx-based dogma draft into a real ((player, game) -> generator) dogma
    // but several generators may exist at one time, each has a reference to f
    // meanwhile, the real dogma may have ended, so they can't refer to the dogma
    // so Rc is used
    // TODO: check if there's some relationship between Rc and Box here
    let rcf = Rc::new(f);
    Box::new(move |player, game| {
        let cloned = Rc::clone(&rcf);
        Gn::new_scoped_local(move |s| {
            let ctx = Context::new(s);
            cloned(player, game, ctx);
            done!()
        })
    })
}

fn demand<F>(f: F) -> DemandFlow
where
    F: for<'a, 'c, 'g> Fn(&'g Player<'c>, &'g Player<'c>, &'g Players<'c>, Context<'a, 'c, 'g>) + 'static,
{
    let rcf = Rc::new(f);
    Box::new(move |player, opponent, game| {
        let cloned = Rc::clone(&rcf);
        Gn::new_scoped_local(move |s| {
            let ctx = Context::new(s);
            cloned(player, opponent, game, ctx);
            done!()
        })
    })
}

pub fn agriculture() -> Vec<Dogma> {
    vec![Dogma::Share(shared(|player, game, mut ctx| {
        let card = ctx.may(player, |ctx| {
            ctx.choose_one_card(player, player.hand().as_vec())
        });
        card.flatten().and_then(|card| {
            game.r#return(player, card);
            game.draw_and_score(player, card.age());
            Some(())
        });
    }))]
}

// inner state
// but need to ensure the execution order, or avoid multiple mutable borrows
// i.e., ensure the first is dropped before the second is created
// also, one dogma will be executed many times by different players
// whether the state is shared is a question
// whether call it only once (lazy???) or once for every execution is a question
// I guess both can work
pub fn oars() -> Vec<Dogma> {
    let transferred: RcCell<bool> = Rc::new(RefCell::new(false));
    let view = Rc::clone(&transferred);
    vec![
        Dogma::Demand(demand(move |player, opponent, game, mut ctx| {
            let card = ctx.choose_one_card(opponent, opponent.hand().has_icon(Icon::Crown));
            card.and_then(|card| {
                // TODO: handle the Result
                game.transfer_card(Place::hand(opponent), Place::score(player), card).unwrap();
                *transferred.borrow_mut() = true;
                Some(())
            });
        })),
        Dogma::Share(shared(move |player, game, _ctx| {
            if !*view.borrow() {
                game.draw(player, 1);
            }
        })),
    ]
}

/*
// OR STRUCT?
// create one in every execution
// but that will cause waste
// so what about RcCell?

struct Oars { transferred: bool }
impl Oars {
}

pub const OARS_1: DemandFlow = demand(|player, opponent, game, ctx| {
    let card = yield ctx.chooseOneCard(opponent, opponent.hand.has(crown));
    card.then(|card| {
        game.transfer(opponent.hand, player.score, card);
        env.set(true);
    });
})
*/

pub fn archery() -> Vec<Dogma> {
    vec![Dogma::Demand(Box::new(|player, opponent, game| {
        Gn::new_scoped_local(move |mut s: Scope<RefChoice, _>| {
            game.draw(opponent, 1);
            let age = opponent.age();
            let cards = s
                .yield_(ExecutionState::new(
                    opponent,
                    Choose::Card {
                        min_num: 1,
                        max_num: Some(1),
                        from: opponent
                            .hand()
                            .as_iter()
                            .filter(|c| c.age() == age)
                            .collect(),
                    },
                ))
                .expect("Generator got None")
                .cards();
            // TODO should handle failure case
            game.transfer_card(Place::hand(opponent), Place::hand(player), cards[0])
                .expect("todo");
            done!()
        })
    }))]
}

pub fn code_of_laws() -> Vec<Dogma> {
    vec![Dogma::Share(Box::new(|player, game| {
        Gn::new_scoped_local(move |mut s: Scope<RefChoice, _>| {
            let opt_card = s
                .yield_(ExecutionState::new(
                    player,
                    Choose::Card {
                        min_num: 0,
                        max_num: Some(1),
                        from: player
                            .hand()
                            .as_iter()
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
            game.tuck(player, card);
            if game.is_splayed(player, card.color(), Splay::Left) {
                done!()
            }
            if s.yield_(ExecutionState::new(player, Choose::Yn))
                .expect("Generator got None")
                .yn()
            {
                game.splay(player, card.color(), Splay::Left);
            }
            done!()
        })
    }))]
}

pub fn optics() -> Vec<Dogma> {
    vec![Dogma::Share(Box::new(|player, game| {
        Gn::new_scoped_local(move |mut s: Scope<RefChoice, _>| {
            let card = game.draw_and_meld(player, 3).unwrap();
            if card.contains(Icon::Crown) {
                game.draw_and_score(player, 4);
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
                game.transfer_card(Place::score(player), Place::score(opponent), card)
                    .unwrap();
                done!()
            }
        })
    }))]
}
