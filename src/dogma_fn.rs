use generator::{done, Gn, Scope};

use crate::{
    action::RefChoice,
    enums::{Icon, Splay},
    flow::{DemandFlow, ShareFlow},
    state::{Choose, ExecutionState},
    structure::Place,
};

/*
struct Context {
    s: Scope<RefChoice, ExecutionState>,
}

impl Context {
    fn new(s: Scope<RefChoice, ExecutionState>) -> Context {
        Context { s }
    }

    fn chooseOneCard(&mut self, player: &'g Player, from: Vec<&'c Card>) -> Option<Card> {
        let cards = self.s
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
        debug_assert!(cards.length() <= 1);
        cards.get(0)
    }

    fn chooseYn(&mut self, player: &'g Player) -> bool {
        self.s.yield_(ExecutionState::new(
            player,
            Choose::Yn
        )).expect("Generator got None").yn()
    }

    fn may<T, F>(&mut self, player: &'g Player, f: F) -> T
    where F: FnOnce() -> T
    {
        self.chooseYn().then(f)
    }
}

fn build_shared(f: F) -> ShareFlow
where F: Fn(Context, &'g Player, &'g Players)
{
    |player, game| {
        Gn::new_scoped_local(move |mut s|
            let ctx = Context::shared(s);
            f(player, game, ctx);
            done!()
        )
    }
}

pub const AGRICULTURE: ShareFlow = shared(|player, game, ctx| {
    let card = ctx.may(player, || ctx.chooseOneCard(player, player.hand));
    card.then(|card| {
        player.return(card);
        player.draw_and_score(card.age());
    });
});

fn f(&'a mut self) -> &'a Thing;

// inner state
// but need to ensure the execution order, or avoid multiple mutable borrows
// i.e., ensure the first is dropped before the second is created
// also, one dogma will be executed many times by different players
// whether the state is shared is a question
pub const OARS: impl Fn() -> Vec<Flow> = || {
    let transferred: RcCell<bool> = RcCell::new(false);
    vec![
        // FnMut
        demand(|player, opponent, game, ctx| {
            let card = ctx.chooseOneCard(opponent, opponent.hand.has(crown));
            card.then(|card| {
                game.transfer(opponent.hand, player.score, card);
                transferred.borrow_mut() = true;
            });
        }),
        shared(|player, game, ctx| {
            if !transferred.borrow() {
                player.draw(1);
            }
        })
    ]
];

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

pub const ARCHERY: DemandFlow = |player, opponent, game| {
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
};

pub const CODE_OF_LAWS: ShareFlow = |player, game| {
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
                        .filter(|card| !player.board().borrow().get_stack(card.color()).is_empty())
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
};

pub const OPTICS: ShareFlow = |player, game| {
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
};
