use generator::{done, Gn, Scope};

use crate::{
    action::RefChoice,
    containers::transfer,
    enums::{Icon, Splay},
    flow::{DemandFlow, ShareFlow},
    state::{Choose, ExecutionState},
};

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
            transfer(&player.score_pile, &opponent.score_pile, card);
            done!()
        }
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
                        .hand
                        .borrow()
                        .as_vec()
                        .into_iter()
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
