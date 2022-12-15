use std::{cell::RefCell, convert::TryInto, rc::Rc};

use generator::{done, Gn, Scope};
use strum::IntoEnumIterator;

use crate::{
    action::RefChoice,
    card::{Card, Dogma},
    enums::{Color, Icon, Splay},
    game::{Players, RcCell},
    player::Player,
    state::{Choose, ExecutionState},
    structure::{Board, Hand, Score},
};

// wrapper of Scope
// which lifetime in Scope???
struct Context<'a, 'c, 'g> {
    s: Scope<'a, RefChoice<'c, 'g>, ExecutionState<'c, 'g>>,
}

impl<'a, 'c, 'g> Context<'a, 'c, 'g> {
    fn new(s: Scope<'a, RefChoice<'c, 'g>, ExecutionState<'c, 'g>>) -> Context<'a, 'c, 'g> {
        Context { s }
    }

    fn yield_(&mut self, player: &'g Player<'c>, choose: Choose<'c>) -> RefChoice<'c, 'g> {
        self.s
            .yield_(ExecutionState::new(player, choose))
            .expect("Generator got None")
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

    fn choose_card_at_most(
        &mut self,
        player: &'g Player<'c>,
        from: Vec<&'c Card>,
        max_num: Option<u8>,
    ) -> Option<Vec<&'c Card>> {
        // choose at least one if possible
        let cards = self
            .s
            .yield_(ExecutionState::new(
                player,
                Choose::Card {
                    min_num: 1,
                    max_num,
                    from,
                },
            ))
            .expect("Generator got None")
            .cards();
        if !cards.is_empty() {
            Some(cards)
        } else {
            None
        }
    }

    fn choose_any_cards_up_to(
        &mut self,
        player: &'g Player<'c>,
        from: Vec<&'c Card>,
        max_num: Option<u8>,
    ) -> Vec<&'c Card> {
        // can choose 0 to max_num cards
        self.s
            .yield_(ExecutionState::new(
                player,
                Choose::Card {
                    min_num: 0,
                    max_num,
                    from,
                },
            ))
            .expect("Generator got None")
            .cards()
    }

    fn choose_cards_exact(
        &mut self,
        player: &'g Player<'c>,
        from: Vec<&'c Card>,
        num: u8,
    ) -> Option<Vec<&'c Card>> {
        let cards = self
            .s
            .yield_(ExecutionState::new(
                player,
                Choose::Card {
                    min_num: num,
                    max_num: Some(num),
                    from,
                },
            ))
            .expect("Generator got None")
            .cards();
        if cards.is_empty() {
            None
        } else {
            Some(cards)
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

    fn may_splay(
        &mut self,
        player: &'g Player<'c>,
        game: &'g Players<'c>,
        color: Color,
        direction: Splay,
    ) -> bool {
        let board = player.board().borrow();
        if board.get_stack(color).len() <= 1 || board.is_splayed(color, direction) {
            return false;
        }
        self.may(player, |_| {
            game.splay(player, color, direction);
        })
        .is_some()
    }
    fn may_splays(
        &mut self,
        player: &'g Player<'c>,
        game: &'g Players<'c>,
        colors: Vec<Color>,
        direction: Splay,
    ) -> bool {
        let board = player.board().borrow();
        let available_top_cards: Vec<_> = colors
            .into_iter()
            .filter(|&color| board.get_stack(color).can_splay(direction))
            .map(|color| board.get_stack(color).top_card().unwrap())
            .collect();
        if available_top_cards.is_empty() {
            return false;
        }
        self.may(player, |ctx| {
            let color = ctx
                .choose_one_card(player, available_top_cards)
                .unwrap()
                .color();
            game.splay(player, color, direction);
        })
        .is_some()
    }
}

fn shared<F>(f: F) -> Dogma
where
    F: for<'a, 'c, 'g> Fn(&'g Player<'c>, &'g Players<'c>, Context<'a, 'c, 'g>) + 'static,
{
    // convert a ctx-based dogma draft into a real ((player, game) -> generator) dogma
    // but several generators may exist at one time, each has a reference to f
    // meanwhile, the real dogma may have ended, so they can't refer to the dogma
    // so Rc is used
    // TODO: check if there's some relationship between Rc and Box here
    let rcf = Rc::new(f);
    Dogma::Share(Box::new(move |player, game| {
        let cloned = Rc::clone(&rcf);
        Gn::new_scoped_local(move |s| {
            let ctx = Context::new(s);
            cloned(player, game, ctx);
            done!()
        })
    }))
}

fn demand<F>(f: F) -> Dogma
where
    F: for<'a, 'c, 'g> Fn(&'g Player<'c>, &'g Player<'c>, &'g Players<'c>, Context<'a, 'c, 'g>)
        + 'static,
{
    let rcf = Rc::new(f);
    Dogma::Demand(Box::new(move |player, opponent, game| {
        let cloned = Rc::clone(&rcf);
        Gn::new_scoped_local(move |s| {
            let ctx = Context::new(s);
            cloned(player, opponent, game, ctx);
            done!()
        })
    }))
}

pub fn pottery() -> Vec<Dogma> {
    vec![
        shared(|player, game, mut ctx| {
            if let Some(cards) = ctx
                .may(player, |ctx| {
                    ctx.choose_card_at_most(player, player.hand().as_vec(), Some(3))
                })
                .flatten()
            {
                let n = cards.len();
                for card in cards {
                    game.r#return(player, card);
                }
                game.draw(player, n.try_into().unwrap());
            }
        }),
        shared(|player, game, _ctx| {
            game.draw(player, 1);
        }),
    ]
}

pub fn tools() -> Vec<Dogma> {
    vec![
        shared(|player, game, mut ctx| {
            // need confirmation of rule, any or exact 3 cards?
            let cards = ctx
                .may(player, |ctx| {
                    ctx.choose_cards_exact(player, player.hand().as_vec(), 3)
                })
                .flatten();
            if cards.is_some() {
                game.draw_and_meld(player, 3);
            }
        }),
        shared(|player, game, mut ctx| {
            let card = ctx
                .may(player, |ctx| {
                    ctx.choose_one_card(player, player.hand().filtered_vec(|&c| c.age() == 3))
                })
                .flatten();
            if card.is_some() {
                game.draw(player, 1);
                game.draw(player, 1);
                game.draw(player, 1);
            }
        }),
    ]
}

// TODO: simplify
pub fn archery() -> Vec<Dogma> {
    vec![demand(|player, opponent, game, mut ctx| {
        game.draw(opponent, 1);
        let age = opponent.age();
        let cards = ctx
            .yield_(
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
            )
            .cards();
        // TODO should handle failure case
        game.transfer_card(opponent.with_id(Hand), player.with_id(Hand), cards[0])
            .expect("todo");
        done!()
    })]
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
        demand(move |player, opponent, game, mut ctx| {
            let card = ctx.choose_one_card(opponent, opponent.hand().has_icon(Icon::Crown));
            if let Some(card) = card {
                // TODO: handle the Result
                game.transfer_card(opponent.with_id(Hand), player.with_id(Score), card)
                    .unwrap();
                *transferred.borrow_mut() = true;
            }
        }),
        shared(move |player, game, _ctx| {
            if !*view.borrow() {
                game.draw(player, 1);
            }
        }),
    ]
}

pub fn agriculture() -> Vec<Dogma> {
    vec![shared(|player, game, mut ctx| {
        let card = ctx.may(player, |ctx| {
            ctx.choose_one_card(player, player.hand().as_vec())
        });
        if let Some(card) = card.flatten() {
            game.r#return(player, card);
            game.draw_and_score(player, card.age());
        }
    })]
}

// TODO: simplify
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

pub fn monotheism() -> Vec<Dogma> {
    vec![
        demand(|player, opponent, game, mut ctx| {
            let available_cards: Vec<_> = opponent
                .board()
                .borrow()
                .top_cards()
                .into_iter()
                .filter(|&card| player.board().borrow().get_stack(card.color()).is_empty())
                .collect();
            // you must transfer a top card in available_cards
            // from your board to my score pile! If you do, draw and tuck a 1!
            let chosen = ctx.choose_one_card(opponent, available_cards);
            if let Some(card) = chosen {
                game.transfer(
                    opponent.with_id(Board),
                    player.with_id(Score),
                    (card.color(), true),
                    (),
                )
                .unwrap();
                game.draw_and_tuck(opponent, 1);
            }
        }),
        shared(|player, game, _ctx| {
            game.draw_and_tuck(player, 1);
        }),
    ]
}

pub fn philosophy() -> Vec<Dogma> {
    vec![
        shared(|player, game, mut ctx| {
            ctx.may_splays(player, game, Color::iter().collect(), Splay::Left);
        }),
        shared(|player, game, mut ctx| {
            if !player.score_pile().as_vec().is_empty() && ctx.choose_yn(player) {
                let card = ctx
                    .choose_one_card(player, player.score_pile().as_vec())
                    .unwrap();
                game.score(player, card);
            }
        }),
    ]
}

// TODO: simplify
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
                game.transfer_card(player.with_id(Score), opponent.with_id(Score), card)
                    .unwrap();
                done!()
            }
        })
    }))]
}
