use std::{cell::RefCell, cmp::min, convert::TryInto, rc::Rc};

use generator::{done, Gn, Scope};
use strum::IntoEnumIterator;

use crate::{
    card::{
        flow::{FlowState, GenResume, GenYield},
        Card,
        Color::{self, *},
        Dogma,
        Icon::*,
        SpecialAchievement,
        Splay::{self, *},
    },
    error::InnResult,
    game::{Players, RcCell},
    player::Player,
    state::{Choose, ExecutionState},
    structure::{Board, Hand, Score},
};

// wrapper of Scope
// which lifetime in Scope???
pub struct Context<'a, 'c, 'g> {
    s: Scope<'a, GenResume<'c, 'g>, GenYield<'c, 'g>>,
}

impl<'a, 'c, 'g> Context<'a, 'c, 'g> {
    pub fn new(s: Scope<'a, GenResume<'c, 'g>, GenYield<'c, 'g>>) -> Context<'a, 'c, 'g> {
        Context { s }
    }

    pub fn into_raw(self) -> Scope<'a, GenResume<'c, 'g>, GenYield<'c, 'g>> {
        self.s
    }

    pub fn yield_(&mut self, player: &'g Player<'c>, choose: Choose<'c>) -> GenResume<'c, 'g> {
        self.s
            .yield_(Ok(ExecutionState::new(player, choose)))
            .expect("Generator got None")
    }

    // TODO: implementation? use is_done or resume or send or ...?
    /// Manual yield from a (local) generator.
    pub fn yield_from(&mut self, mut gen: FlowState<'c, 'g>) {
        let mut res = gen.resume();
        while let Some(request) = res {
            let choice = self.s.yield_(request).expect("Generator got None");
            gen.set_para(choice);
            res = gen.resume();
        }
    }

    pub fn choose_one_card(
        &mut self,
        player: &'g Player<'c>,
        from: Vec<&'c Card>,
    ) -> Option<&'c Card> {
        // MAYFIXED: TODO: cards not enough, etc?
        self.yield_(
            player,
            Choose::Card {
                min_num: 1,
                max_num: Some(1),
                from,
            },
        )
        .card()
    }

    pub fn may_choose_one_card(
        &mut self,
        player: &'g Player<'c>,
        from: Vec<&'c Card>,
    ) -> Option<&'c Card> {
        // TODO: use what form? min_num = 0 or yn.then(1 card) or other?
        let cards = self
            .yield_(
                player,
                Choose::Card {
                    min_num: 0,
                    max_num: Some(1),
                    from,
                },
            )
            .cards()?;
        debug_assert!(cards.len() <= 1);
        cards.first().copied()
    }

    pub fn choose_card_at_most(
        &mut self,
        player: &'g Player<'c>,
        from: Vec<&'c Card>,
        max_num: Option<usize>,
    ) -> Option<Vec<&'c Card>> {
        // choose at least one if possible
        self.yield_(
            player,
            Choose::Card {
                min_num: 1,
                max_num,
                from,
            },
        )
        .cards()
    }

    pub fn choose_any_cards_up_to(
        &mut self,
        player: &'g Player<'c>,
        from: Vec<&'c Card>,
        max_num: Option<usize>,
    ) -> Vec<&'c Card> {
        // can choose 0 to max_num cards
        self.yield_(
            player,
            Choose::Card {
                min_num: 0,
                max_num,
                from,
            },
        )
        .cards()
        .expect("The actor can choose 0 cards, so there should always be valid action.")
    }

    pub fn choose_cards_exact(
        &mut self,
        player: &'g Player<'c>,
        from: Vec<&'c Card>,
        num: usize,
    ) -> Option<Vec<&'c Card>> {
        self.yield_(
            player,
            Choose::Card {
                min_num: num,
                max_num: Some(num),
                from,
            },
        )
        .cards()
    }

    pub fn choose_yn(&mut self, player: &'g Player<'c>) -> bool {
        self.yield_(player, Choose::Yn)
            .yn()
            .expect("Actors should always have valid actions when choosing yes or no.")
    }

    pub fn may<T, F>(&mut self, player: &'g Player<'c>, f: F) -> InnResult<Option<T>>
    where
        F: FnOnce(&mut Context<'a, 'c, 'g>) -> InnResult<T>,
    {
        self.choose_yn(player).then(|| f(self)).transpose()
    }

    pub fn may_splay(
        &mut self,
        player: &'g Player<'c>,
        game: &'g Players<'c>,
        color: Color,
        direction: Splay,
    ) -> InnResult<bool> {
        if player.can_splay(color, direction) {
            Ok(self
                .may(player, |_| game.splay(player, color, direction))?
                .is_some())
        } else {
            Ok(false)
        }
    }

    pub fn may_splays(
        &mut self,
        player: &'g Player<'c>,
        game: &'g Players<'c>,
        colors: Vec<Color>,
        direction: Splay,
    ) -> InnResult<bool> {
        let available_top_cards: Vec<_> = colors
            .into_iter()
            .filter(|&color| player.can_splay(color, direction))
            .map(|color| player.stack(color).top_card().unwrap())
            .collect();
        if available_top_cards.is_empty() {
            return Ok(false);
        }
        Ok(self
            .may(player, |ctx| {
                let color = ctx
                    .choose_one_card(player, available_top_cards)
                    .unwrap()
                    .color();
                game.splay(player, color, direction)
            })?
            .is_some())
    }
}

pub fn mk_execution<'c, 'g, F>(f: F) -> FlowState<'c, 'g>
where
    F: for<'a> FnOnce(&mut Context<'a, 'c, 'g>) -> InnResult<()> + 'g,
{
    Gn::new_scoped_local(|s| {
        let mut ctx = Context::new(s);
        if let Err(e) = f(&mut ctx) {
            ctx.into_raw().yield_(Err(e));
            panic!(
                "Error happened but an progress of execution has been made!\n\
                So the same execution process can't be touched.\n\
                This can be caused by unknown internal error OR winning."
            )
        };
        done!()
    })
}

fn shared<F>(f: F) -> Dogma
where
    F: for<'a, 'c, 'g> Fn(
            &'g Player<'c>,
            &'g Players<'c>,
            &mut Context<'a, 'c, 'g>,
        ) -> InnResult<()>
        + 'static,
{
    // convert a ctx-based dogma draft into a real ((player, game) -> generator) dogma
    // but several generators may exist at one time, each has a reference to f
    // meanwhile, the real dogma may have ended, so they can't refer to the dogma
    // so Rc is used
    // TODO: check if there's some relationship between Rc and Box here
    let rcf = Rc::new(f);
    Dogma::Share(Box::new(move |player, game| {
        let cloned = Rc::clone(&rcf);
        mk_execution(move |ctx| cloned(player, game, ctx))
    }))
}

fn demand<F>(f: F) -> Dogma
where
    F: for<'a, 'c, 'g> Fn(
            &'g Player<'c>,
            &'g Player<'c>,
            &'g Players<'c>,
            &mut Context<'a, 'c, 'g>,
        ) -> InnResult<()>
        + 'static,
{
    let rcf = Rc::new(f);
    Dogma::Demand(Box::new(move |player, opponent, game| {
        let cloned = Rc::clone(&rcf);
        mk_execution(move |ctx| cloned(player, opponent, game, ctx))
    }))
}

pub fn pottery() -> Vec<Dogma> {
    vec![
        shared(|player, game, ctx| {
            let cards = ctx.choose_any_cards_up_to(player, player.hand().to_vec(), Some(3));
            if !cards.is_empty() {
                let n = cards.len();
                for card in cards {
                    game.r#return(player, card)?;
                }
                game.draw(player, n.try_into().unwrap())?;
            }
            Ok(())
        }),
        shared(|player, game, _ctx| {
            game.draw(player, 1)?;
            Ok(())
        }),
    ]
}

pub fn tools() -> Vec<Dogma> {
    vec![
        shared(|player, game, ctx| {
            // need confirmation of rule, any or exact 3 cards?
            let cards = ctx
                .may(player, |ctx| {
                    Ok(ctx.choose_cards_exact(player, player.hand().to_vec(), 3))
                })?
                .flatten();
            if cards.is_some() {
                game.draw_and_meld(player, 3)?;
            }
            Ok(())
        }),
        shared(|player, game, ctx| {
            let card = ctx
                .may(player, |ctx| {
                    Ok(ctx.choose_one_card(player, player.hand().filtered_vec(|&c| c.age() == 3)))
                })?
                .flatten();
            if card.is_some() {
                game.draw(player, 1)?;
                game.draw(player, 1)?;
                game.draw(player, 1)?;
            }
            Ok(())
        }),
    ]
}

pub fn archery() -> Vec<Dogma> {
    vec![demand(|player, opponent, game, ctx| {
        game.draw(opponent, 1)?;
        let age = opponent
            .hand()
            .iter()
            .max_by_key(|c| c.age())
            .expect("After drawn a 1, opponent should have at least one card.")
            .age();
        let card = ctx
            .choose_one_card(opponent, opponent.hand().filtered_vec(|c| c.age() == age))
            .expect("After drawn a 1, opponent should have at least one card.");
        game.transfer_card(&opponent.with_id(Hand), &player.with_id(Hand), card)?;
        Ok(())
    })]
}

pub fn metalworking() -> Vec<Dogma> {
    vec![shared(|player, game, _ctx| {
        loop {
            // TODO: draw and reveal
            let card = game.draw(player, 1)?;
            if card.contains(Castle) {
                game.score(player, card)?;
            } else {
                break Ok(());
            }
        }
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
        demand(move |player, opponent, game, ctx| {
            let card = ctx.choose_one_card(opponent, opponent.hand().has_icon(Crown));
            if let Some(card) = card {
                // MAYFIXED: TODO: handle the Result
                game.transfer_card(&opponent.with_id(Hand), &player.with_id(Score), card)?;
                game.draw(opponent, 1)?;
                *transferred.borrow_mut() = true;
            }
            Ok(())
        }),
        shared(move |player, game, _ctx| {
            if !*view.borrow() {
                game.draw(player, 1)?;
            }
            Ok(())
        }),
    ]
}

pub fn clothing() -> Vec<Dogma> {
    vec![
        shared(|player, game, ctx| {
            let card = ctx.choose_one_card(
                player,
                player
                    .hand()
                    .filtered_vec(|c| player.stack(c.color()).is_empty()),
            ); // make this a separate statement to avoid hand borrowing issue
            if let Some(card) = card {
                game.meld(player, card)?;
            }
            Ok(())
        }),
        shared(|player, game, _ctx| {
            let num_scores = Color::iter()
                .filter(|&color| {
                    !player.stack(color).is_empty()
                        && game
                            .opponents_of(player.id())
                            .all(|op| op.stack(color).is_empty())
                })
                .count();
            for _ in 0..num_scores {
                game.draw_and_score(player, 1)?;
            }
            Ok(())
        }),
    ]
}

pub fn agriculture() -> Vec<Dogma> {
    vec![shared(|player, game, ctx| {
        let card = ctx.may(player, |ctx| {
            Ok(ctx.choose_one_card(player, player.hand().to_vec()))
        })?;
        if let Some(card) = card.flatten() {
            game.r#return(player, card)?;
            game.draw_and_score(player, card.age() + 1)?;
        }
        Ok(())
    })]
}

pub fn domestication() -> Vec<Dogma> {
    vec![shared(|player, game, ctx| {
        let min_age = player.hand().iter().map(|c| c.age()).min();
        if let Some(min_age) = min_age {
            let card = ctx
                .choose_one_card(player, player.hand().filtered_vec(|c| c.age() == min_age))
                .expect(
                    "There's a min age in player's hand, so there should be \
                    a card that can be chosen.",
                );
            game.meld(player, card)?;
        }
        game.draw(player, 1)?;
        Ok(())
    })]
}

pub fn masonry() -> Vec<Dogma> {
    vec![shared(|player, game, ctx| {
        let to_melds = ctx.choose_any_cards_up_to(player, player.hand().has_icon(Castle), None);
        let len = to_melds.len();
        for card in to_melds {
            game.meld(player, card)?;
        }
        if len >= 4 {
            game.achieve_if_available(player, &SpecialAchievement::Monument.into())?;
        }
        Ok(())
    })]
}

pub fn city_states() -> Vec<Dogma> {
    vec![demand(|player, opponent, game, ctx| {
        if opponent.board().icon_count()[&Castle] >= 4 {
            if let Some(card) = ctx.choose_one_card(
                opponent,
                opponent
                    .board()
                    .top_cards()
                    .into_iter()
                    .filter(|c| c.contains(Castle))
                    .collect(),
            ) {
                game.transfer(&opponent.with_id(Board), &player.with_id(Board), card, true)?;
                game.draw(opponent, 1)?;
            }
        }
        Ok(())
    })]
}

pub fn code_of_laws() -> Vec<Dogma> {
    vec![shared(|player, game, ctx| {
        let opt_card = ctx.may_choose_one_card(
            player,
            player
                .hand()
                .filtered_vec(|card| !player.stack(card.color()).is_empty()),
        );
        let card = match opt_card {
            Some(c) => c,
            None => return Ok(()),
        };
        game.tuck(player, card)?;
        // TODO: use may_splay, and/or implement may_splay use this method?
        if player.can_splay(card.color(), Left) && ctx.choose_yn(player) {
            game.splay(player, card.color(), Left)?;
        }
        Ok(())
    })]
}

pub fn mysticism() -> Vec<Dogma> {
    vec![shared(|player, game, _ctx| {
        let card = game.draw(player, 1)?;
        if !player.stack(card.color()).is_empty() {
            game.meld(player, card)?;
            game.draw(player, 1)?;
        }
        Ok(())
    })]
}

pub fn monotheism() -> Vec<Dogma> {
    vec![
        demand(|player, opponent, game, ctx| {
            let available_cards: Vec<_> = opponent
                .board()
                .top_cards()
                .into_iter()
                .filter(|&card| player.stack(card.color()).is_empty())
                .collect();
            // you must transfer a top card in available_cards
            // from your board to my score pile! If you do, draw and tuck a 1!
            let chosen = ctx.choose_one_card(opponent, available_cards);
            if let Some(card) = chosen {
                game.transfer(
                    &opponent.with_id(Board),
                    &player.with_id(Score),
                    &(card.color(), true),
                    (),
                )
                .unwrap();
                game.draw_and_tuck(opponent, 1)?;
            }
            Ok(())
        }),
        shared(|player, game, _ctx| {
            game.draw_and_tuck(player, 1)?;
            Ok(())
        }),
    ]
}

pub fn philosophy() -> Vec<Dogma> {
    vec![
        shared(|player, game, ctx| {
            ctx.may_splays(player, game, Color::iter().collect(), Left)?;
            Ok(())
        }),
        shared(|player, game, ctx| {
            if !player.score_pile().to_vec().is_empty() && ctx.choose_yn(player) {
                let card = ctx
                    .choose_one_card(player, player.score_pile().to_vec())
                    .unwrap();
                game.score(player, card)?;
            }
            Ok(())
        }),
    ]
}

pub fn optics() -> Vec<Dogma> {
    vec![shared(|player, game, ctx| {
        let card = game.draw_and_meld(player, 3)?;
        if card.contains(Crown) {
            game.draw_and_score(player, 4)?;
            Ok(())
        } else {
            let card = match ctx.choose_one_card(player, player.score_pile().to_vec()) {
                Some(card) => card,
                None => return Ok(()),
            };
            // TODO: can only choose players that have lower score than you
            let opponent = ctx.yield_(player, Choose::Opponent).player().expect(
                "Actors should always have valid actions when choosing an opponent currently, \
                as choose_players_from() has not yet been implemented.",
            );
            game.transfer_card(&player.with_id(Score), &opponent.with_id(Score), card)?;
            Ok(())
        }
    })]
}

pub fn anatomy() -> Vec<Dogma> {
    vec![demand(|_player, opponent, game, ctx| {
        if let Some(score_card) = ctx.choose_one_card(opponent, opponent.hand().to_vec()) {
            game.return_from(opponent, score_card, &opponent.with_id(Score))?;
            if let Some(board_card) = ctx.choose_one_card(
                opponent,
                opponent
                    .board()
                    .top_cards()
                    .into_iter()
                    .filter(|c| c.age() == score_card.age())
                    .collect(),
            ) {
                game.return_from(opponent, board_card, &opponent.with_id(Board))?;
            }
        }
        Ok(())
    })]
}

pub fn enterprise() -> Vec<Dogma> {
    vec![
        demand(|player, opponent, game, ctx| {
            if let Some(card) = ctx.choose_one_card(
                opponent,
                opponent
                    .board()
                    .top_cards()
                    .into_iter()
                    .filter(|c| c.color() != Purple && c.contains(Crown))
                    .collect(),
            ) {
                game.transfer(&opponent.with_id(Board), &player.with_id(Board), card, true)?;
                game.draw_and_meld(opponent, 4)?;
            }
            Ok(())
        }),
        shared(|player, game, ctx| {
            ctx.may_splay(player, game, Green, Right)?;
            Ok(())
        }),
    ]
}

pub fn reformation() -> Vec<Dogma> {
    vec![
        shared(|player, game, ctx| {
            let num_leaves = player.board().icon_count()[&Leaf];
            let num_cards = min(num_leaves % 2, player.hand().to_vec().len());
            if num_cards >= 1 && ctx.choose_yn(player) {
                let cards = ctx
                    .choose_cards_exact(player, player.hand().to_vec(), num_cards)
                    .expect("Player should be able to choose cards of computed number.");
                for card in cards {
                    game.tuck(player, card)?;
                }
            }
            Ok(())
        }),
        shared(|player, game, ctx| {
            ctx.may_splays(player, game, vec![Yellow, Purple], Right)?;
            Ok(())
        }),
    ]
}

pub fn computers() -> Vec<Dogma> {
    vec![
        shared(|player, game, ctx| {
            ctx.may_splays(player, game, vec![Red, Green], Up)?;
            Ok(())
        }),
        shared(|player, game, ctx| {
            let card = game.draw_and_meld(player, 10)?;
            ctx.yield_from(game.execute_shared_alone(player, card));
            Ok(())
        }),
    ]
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        action::{Action, NoRefStep},
        card::default_cards,
        card_pile::MainCardPile,
        game::GameConfig,
        logger::{Logger, Observer},
        player::PlayerBuilder,
    };

    #[test]
    fn clothing_borrowing() {
        let clothing = default_cards::clothing();
        let agriculture = default_cards::agriculture();
        let mut game = GameConfig::new(vec![&clothing, &agriculture])
            .player(
                0,
                PlayerBuilder::default()
                    .board(vec![&clothing])
                    .hand(vec![&agriculture]),
            )
            .build();
        game.step(Action::Step(NoRefStep::Execute("Clothing".to_owned())))
            .unwrap();
    }

    #[test]
    fn domestication_borrowing() {
        let domestication = default_cards::domestication();
        let agriculture = default_cards::agriculture();
        let clothing = default_cards::clothing();
        let mut game = GameConfig::new(vec![&domestication, &agriculture, &clothing])
            .main_pile(MainCardPile::builder().draw_deck(vec![&clothing]).build())
            .player(
                0,
                PlayerBuilder::default()
                    .board(vec![&domestication])
                    .hand(vec![&agriculture]),
            )
            .build();
        game.step(Action::Step(NoRefStep::Execute("Domestication".to_owned())))
            .unwrap();
        assert!(vec![&domestication, &agriculture]
            .into_iter()
            .all(|card| game.observe(0).main_player.board.contains(card)));
        assert!(game.observe(0).main_player.hand.contains(&&clothing))
    }

    #[test]
    fn enterprise_borrowing<'a>() {
        let enterprise = default_cards::enterprise();
        let optics = default_cards::optics();
        let anatomy = default_cards::anatomy();
        let mut logger = Logger::new();
        logger.start(Default::default()); // TODO: make recording starting card order "optional"
        let logger: Rc<RefCell<dyn Observer>> = Rc::new(RefCell::new(logger));
        let mut game = GameConfig::new(vec![&optics, &enterprise, &anatomy])
            .main_pile(MainCardPile::builder().draw_deck(vec![&anatomy]).build())
            .players(vec![
                PlayerBuilder::default().board(vec![&enterprise]),
                PlayerBuilder::default().board(vec![&optics]),
            ])
            .observe(&logger)
            .build();
        game.step(Action::Step(NoRefStep::Execute("Enterprise".to_owned())))
            .unwrap();
    }
}
