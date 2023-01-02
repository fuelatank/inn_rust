use std::cell::RefCell;
use std::rc::Rc;

use generator::{done, Gn};
use ouroboros::self_referencing;
use strum::IntoEnumIterator;

use crate::{
    action::{Action, NoRefChoice, NoRefStepAction, RefAction, RefStepAction},
    auto_achieve::{AchievementManager, WinByAchievementChecker},
    card::{Achievement, Card, SpecialAchievement},
    card_pile::MainCardPile,
    containers::{Addable, BoxCardSet, CardSet, Removeable, VecSet},
    dogma_fn::mk_execution,
    enums::{Color, Splay},
    error::{InnResult, InnovationError, WinningSituation},
    flow::{Dogma, FlowState},
    logger::{FnPureObserver, Game, Item, Logger, Operation, SimpleOp, Subject},
    observation::{EndObservation, GameState, ObsType, Observation, SingleAchievementView},
    player::{Player, PlayerBuilder},
    state::{Choose, State},
    structure::{
        AddToGame, Board, Hand, MainCardPile as MainCardPile_, Place, RemoveFromGame, Score,
        TestRemoveFromGame,
    },
    turn::{LoggingTurn, Turn, TurnBuilder},
};

pub type RcCell<T> = Rc<RefCell<T>>;
pub type PlayerId = usize;

pub struct Players<'c> {
    cards: Vec<&'c Card>,
    logger: Subject<'c>,
    main_card_pile: RcCell<MainCardPile<'c>>,
    players: Vec<Player<'c>>,
}

impl<'c> Players<'c> {
    pub fn empty() -> Players<'c> {
        Players {
            cards: Vec::new(),
            logger: Subject::new(),
            main_card_pile: Rc::new(RefCell::new(MainCardPile::empty())),
            players: vec![],
        }
    }

    pub fn new<C>(
        num_players: usize,
        cards: Vec<&'c Card>,
        logger: RcCell<Logger<'c>>,
        first_player: PlayerId,
    ) -> Players<'c>
    where
        C: CardSet<'c, Card> + Default + 'c,
    {
        let pile = MainCardPile::new_init(cards.clone(), SpecialAchievement::iter().collect());
        Players::from_builders(
            cards,
            pile,
            logger,
            (0..num_players)
                .map(|_| PlayerBuilder::new::<C>())
                .collect(),
            first_player,
        )
    }

    pub fn from_builders(
        cards: Vec<&'c Card>,
        main_pile: MainCardPile<'c>,
        logger: RcCell<Logger<'c>>,
        players: Vec<PlayerBuilder<'c>>,
        first_player: PlayerId,
    ) -> Players<'c> {
        let pile = Rc::new(RefCell::new(main_pile));
        let mut subject = Subject::new();
        subject.register_internal_owned(AchievementManager::new(
            SpecialAchievement::iter()
                .filter(|&sa| {
                    pile.borrow()
                        .has_achievement(&SingleAchievementView::Special(sa))
                })
                .collect(),
            first_player,
        ));
        subject.register_internal_owned(WinByAchievementChecker);
        // Should logger cards be initialized here, or in other methods?
        logger.borrow_mut().start(pile.borrow().contents());
        subject.register_external_owned(FnPureObserver::new(move |ev| {
            logger.borrow_mut().log(ev.clone())
        }));
        Players {
            cards,
            logger: subject,
            main_card_pile: Rc::clone(&pile),
            players: players
                .into_iter()
                .enumerate()
                .map(|(id, pb)| pb.build(id))
                .collect(),
        }
    }

    pub fn find_card(&self, name: &str) -> &'c Card {
        self.cards
            .iter()
            .find(|&&card| card.name() == name)
            .unwrap_or_else(|| panic!("no card named {}", name))
    }

    pub fn add_player(
        &mut self,
        hand: BoxCardSet<'c>,
        score_pile: BoxCardSet<'c>,
        achievements: VecSet<Achievement<'c>>,
    ) {
        let id = self.players.len();
        self.players
            .push(Player::new(id, hand, score_pile, achievements))
    }

    pub fn num_players(&self) -> usize {
        self.players.len()
    }

    pub fn players(&self) -> Vec<&Player<'c>> {
        self.players.iter().collect()
    }

    pub fn player_at(&self, id: PlayerId) -> &Player<'c> {
        &self.players[id]
    }

    /// Creates an iterator containing all players once, starting from the
    /// `main_player_id`th player.
    ///
    /// `main_player_id` can be any number, and the index will be rounded for convenience.
    pub fn players_from(&self, main_player_id: PlayerId) -> impl Iterator<Item = &Player<'c>> {
        (0..self.players.len())
            .map(move |i| &self.players[(i + main_player_id) % self.players.len()])
    }

    pub fn ids_from(&self, main_player_id: PlayerId) -> impl Iterator<Item = PlayerId> {
        let len = self.players.len();
        (0..len).map(move |i| (i + main_player_id) % len)
    }

    pub fn main_card_pile(&self) -> &RcCell<MainCardPile<'c>> {
        &self.main_card_pile
    }

    pub fn draw<'g>(&'g self, player: &'g Player<'c>, age: u8) -> InnResult<&'c Card> {
        // transfer(Rc::clone(&self.main_pile), &self.hand, &age)
        self.transfer(MainCardPile_, player.with_id(Hand), age, ())
            .and_then(|r| {
                self.logger
                    .operate(Operation::SimpleOp(SimpleOp::Draw, player.id(), r), self)?;
                Ok(r)
            })
    }

    pub fn draw_and_meld<'g>(&'g self, player: &'g Player<'c>, age: u8) -> InnResult<&'c Card> {
        // transfer(Rc::clone(&self.main_pile), &self.main_board, &age)
        self.transfer(MainCardPile_, player.with_id(Board), age, true)
            .and_then(|r| {
                self.logger.operate(
                    Operation::SimpleOp(SimpleOp::DrawAndMeld, player.id(), r),
                    self,
                )?;
                Ok(r)
            })
    }

    pub fn draw_and_score<'g>(&'g self, player: &'g Player<'c>, age: u8) -> InnResult<&'c Card> {
        // transfer(Rc::clone(&self.main_pile), &self.score_pile, &age)
        self.transfer(MainCardPile_, player.with_id(Score), age, ())
            .and_then(|r| {
                self.logger.operate(
                    Operation::SimpleOp(SimpleOp::DrawAndScore, player.id(), r),
                    self,
                )?;
                Ok(r)
            })
    }

    pub fn draw_and_tuck<'g>(&'g self, player: &'g Player<'c>, age: u8) -> InnResult<&'c Card> {
        self.transfer(MainCardPile_, player.with_id(Board), age, false)
            .and_then(|r| {
                self.logger.operate(
                    Operation::SimpleOp(SimpleOp::DrawAndTuck, player.id(), r),
                    self,
                )?;
                Ok(r)
            })
    }

    pub fn meld<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> InnResult<&'c Card> {
        // transfer(&self.hand, &self.main_board, card)
        self.transfer(player.with_id(Hand), player.with_id(Board), card, true)
            .and_then(|r| {
                self.logger
                    .operate(Operation::SimpleOp(SimpleOp::Meld, player.id(), r), self)?;
                Ok(r)
            })
    }

    pub fn score<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> InnResult<&'c Card> {
        // transfer(&self.hand, &self.score_pile, card)
        self.transfer(player.with_id(Hand), player.with_id(Score), card, ())
            .and_then(|r| {
                self.logger
                    .operate(Operation::SimpleOp(SimpleOp::Score, player.id(), r), self)?;
                Ok(r)
            })
    }

    pub fn tuck<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> InnResult<&'c Card> {
        // transfer(&self.hand, &self.main_board, card)
        self.transfer(player.with_id(Hand), player.with_id(Board), card, false)
            .and_then(|r| {
                self.logger
                    .operate(Operation::SimpleOp(SimpleOp::Tuck, player.id(), r), self)?;
                Ok(r)
            })
    }

    pub fn splay<'g>(
        &'g self,
        player: &'g Player<'c>,
        color: Color,
        direction: Splay,
    ) -> InnResult<()> {
        // error when not able to splay?
        player
            .board()
            .borrow_mut()
            .get_stack_mut(color)
            .splay(direction);
        self.logger
            .operate(Operation::Splay(player.id(), color, direction), self)?;
        Ok(())
    }

    pub fn is_splayed<'g>(
        &'g self,
        player: &'g Player<'c>,
        color: Color,
        direction: Splay,
    ) -> bool {
        player.board().borrow().is_splayed(color, direction)
    }

    pub fn r#return<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> InnResult<&'c Card> {
        // transfer(&self.hand, Rc::clone(&self.main_pile), card)
        self.transfer(player.with_id(Hand), MainCardPile_, card, ())
            .and_then(|r| {
                self.logger
                    .operate(Operation::SimpleOp(SimpleOp::Return, player.id(), r), self)?;
                Ok(r)
            })
    }

    pub fn has_achievement(&self, view: &SingleAchievementView) -> bool {
        self.main_card_pile.borrow().has_achievement(view)
    }

    pub fn try_achieve<'g>(
        &'g self,
        player: &'g Player<'c>,
        view: &SingleAchievementView,
    ) -> InnResult<()> {
        match self.main_card_pile.borrow_mut().remove(view) {
            Some(achievement) => {
                player.achievements_mut().add(achievement);
                self.logger
                    .operate(Operation::Achieve(player.id(), view.clone()), self)?;
                Ok(())
            }
            None => Err(InnovationError::CardNotFound),
        }
    }

    pub fn execute<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> FlowState<'c, 'g> {
        Gn::new_scoped_local(move |mut s| {
            let id = player.id();
            let main_icon = card.main_icon();
            let main_icon_count = player.board().borrow().icon_count()[&main_icon];
            // check eligible players before actual execution
            let players_from_next = self.players_from(id + 1);
            let can_be_shared: Vec<_> = players_from_next
                .map(|p| p.board().borrow().icon_count()[&main_icon] >= main_icon_count)
                .collect();
            // execution
            for dogma in card.dogmas() {
                match dogma {
                    Dogma::Share(flow) => {
                        // filter out ineligible players
                        for player in self
                            .players_from(id + 1)
                            .zip(can_be_shared.iter())
                            .filter_map(|(p, mask)| mask.then_some(p))
                        {
                            let mut gen = flow(player, self);

                            // s.yield_from(gen); but with or(card)
                            let mut state = gen.resume();
                            while let Some(st) = state {
                                let a = s
                                    .yield_(
                                        st.map(|st| st.or(card))
                                            .map_err(|e| e.or_set_current_player(player.id())),
                                    )
                                    .expect("Generator got None");
                                gen.set_para(a);
                                state = gen.resume();
                            }
                        }
                    }
                    Dogma::Demand(flow) => {
                        // filter out ineligible players
                        for player in self
                            .players_from(id)
                            .skip(1)
                            .zip(can_be_shared.iter())
                            .filter_map(|(p, mask)| (!mask).then_some(p))
                        {
                            let mut gen = flow(self.player_at(id), player, self);
                            // s.yield_from(gen); but with or(card)
                            let mut state = gen.resume();
                            while let Some(st) = state {
                                let a = s
                                    .yield_(
                                        st.map(|st| st.or(card))
                                            .map_err(|e| e.or_set_current_player(player.id())),
                                    )
                                    .expect("Generator got None");
                                gen.set_para(a);
                                state = gen.resume();
                            }
                        }
                    }
                }
            }
            done!()
        })
    }

    pub fn win<'g>(&'g self, player: &'g Player<'c>) -> InnResult<()> {
        Err(InnovationError::Win {
            current_player: None,
            situation: WinningSituation::SomeOne(player.id()),
        })
    }

    pub fn transfer<Fr, To, RP, AP>(
        &self,
        from: Fr,
        to: To,
        remove_param: RP,
        add_param: AP,
    ) -> InnResult<&'c Card>
    where
        Fr: RemoveFromGame<'c, RP> + Into<Place>,
        To: AddToGame<'c, AP> + Into<Place>,
    {
        let card = from.remove_from(self, remove_param);
        card.map(|card| {
            to.add_to(card, self, add_param);
            // MAYRESOLVED: TODO: this does not allow observers to perform operations (log)
            // log event, after actual operation, to ensure that observers act after operation
            self.logger
                .operate(Operation::Transfer(from.into(), to.into(), card), self)?;
            Ok(card)
        })
        .and_then(|r| r)
    }

    pub fn transfer_card<Fr, To>(&self, from: Fr, to: To, card: &'c Card) -> InnResult<()>
    where
        Fr: RemoveFromGame<'c, &'c Card> + Into<Place>,
        To: AddToGame<'c, ()> + Into<Place>,
    {
        self.transfer(from, to, card, ()).map(|_| ())
    }

    pub fn exchange<Fr, To>(
        &self,
        place1: Fr,
        place2: To,
        cards12: Vec<&'c Card>,
        cards21: Vec<&'c Card>,
    ) -> InnResult<()>
    where
        Fr: TestRemoveFromGame<'c, &'c Card> + AddToGame<'c, ()> + Into<Place>,
        To: TestRemoveFromGame<'c, &'c Card> + AddToGame<'c, ()> + Into<Place>,
    {
        // first check if this operation can work
        // if not, return the first Err detected
        // check first to avoid half execution when found an Err
        for card in cards12.clone() {
            place1.test_remove(self, card)?;
        }
        for card in cards21.clone() {
            place2.test_remove(self, card)?;
        }
        // after ensured no chance of Err, we can perform the operation
        for card in cards12.clone() {
            place1
                .remove_from(self, card)
                .expect("remove_from() should be consistent with test_remove().");
            place2.add_to(card, self, ());
        }
        for card in cards21.clone() {
            place2
                .remove_from(self, card)
                .expect("remove_from() should be consistent with test_remove().");
            place1.add_to(card, self, ());
        }
        self.logger.operate(
            Operation::Exchange(place1.into(), place2.into(), cards12, cards21),
            self,
        )?;
        Ok(())
    }

    pub fn start_choice<'g>(&'g self) -> FlowState<'c, 'g> {
        mk_execution(move |ctx| {
            for player in self.players_from(0) {
                self.draw(player, 1)?;
                self.draw(player, 1)?;
            }
            for player in self.players_from(0) {
                let card = ctx.choose_one_card(player, player.hand().as_vec()).expect("Already checked, and all players have two cards, so they can always choose one");
                self.meld(player, card)?;
            }
            Ok(())
        })
    }

    pub fn notify(&self, item: Item<'c>) -> InnResult<()> {
        self.logger.notify(item, self)
    }
}

#[self_referencing]
pub struct OuterGame<'c> {
    players: Players<'c>,
    #[borrows(players)]
    players_ref: &'this Players<'c>,
    #[borrows(players)]
    #[covariant]
    turn: LoggingTurn<'c, 'this>,
    logger: RcCell<Logger<'c>>,
    #[borrows()]
    #[covariant]
    state: State<'c, 'this>,
    next_action_type: ObsType<'c>,
}

impl<'c> OuterGame<'c> {
    pub fn init<C>(num_players: usize, cards: Vec<&'c Card>) -> OuterGame<'c>
    where
        C: CardSet<'c, Card> + Default + 'c,
    {
        let logger = Rc::new(RefCell::new(Logger::new()));
        // TODO: structure not clear
        let turn = Turn::new(num_players);
        OuterGameBuilder {
            players: Players::new::<C>(
                num_players,
                cards,
                Rc::clone(&logger),
                turn.current_player(),
            ),
            players_ref_builder: |players| players,
            turn_builder: |players| LoggingTurn::new(turn, players),
            logger,
            state: State::Main,
            next_action_type: ObsType::Main,
        }
        .build()
    }

    pub fn config(
        cards: Vec<&'c Card>,
        main_pile: MainCardPile<'c>,
        players: Vec<PlayerBuilder<'c>>,
        turn: TurnBuilder,
    ) -> OuterGame<'c> {
        let logger = Rc::new(RefCell::new(Logger::new()));
        // TODO: structure not clear
        let turn = turn.build(players.len());
        OuterGameBuilder {
            players: Players::from_builders(
                cards,
                main_pile,
                Rc::clone(&logger),
                players,
                turn.current_player(),
            ),
            players_ref_builder: |players| players,
            turn_builder: |players| LoggingTurn::new(turn, players),
            logger,
            state: State::Main,
            next_action_type: ObsType::Main,
        }
        .build()
    }

    pub fn start(&mut self) -> InnResult<GameState> {
        self.with_mut(|fields| {
            *fields.state = State::Executing((*fields.players_ref).start_choice());
        });
        self.resume_execution()
    }

    fn is_available_action(&self, action: &Action) -> bool {
        self.with(|fields| match (action, fields.next_action_type) {
            (Action::Step(step), ObsType::Main) => {
                if let NoRefStepAction::Draw = step {
                    true
                } else {
                    let players = fields.players;
                    let player = &players.players[fields.turn.player_id()];
                    match step {
                        NoRefStepAction::Meld(c) => {
                            player.hand().as_vec().contains(&players.find_card(c))
                        }
                        NoRefStepAction::Achieve(age) => {
                            player.age() >= *age
                                && player.total_score() >= 5 * (*age as usize)
                                && players.has_achievement(&SingleAchievementView::Normal(*age))
                        }
                        NoRefStepAction::Execute(c) => {
                            player.board().borrow().contains(players.find_card(c))
                        }
                        _ => panic!("just checked, action can't be Draw"),
                    }
                }
            }
            (Action::Executing(choice), ObsType::Executing(obs)) => match (choice, &obs.state) {
                (
                    NoRefChoice::Card(cards),
                    Choose::Card {
                        min_num,
                        max_num,
                        from,
                    },
                ) => {
                    let len = cards.len() as u8;
                    len >= *min_num
                        && match max_num {
                            Some(max) => len <= *max,
                            None => true,
                        }
                        && {
                            // performance?
                            // check if `cards` is a subset of `from`
                            cards
                                .iter()
                                .all(|name| from.iter().any(|c| c.name() == name))
                        }
                }
                (NoRefChoice::Opponent(_), Choose::Opponent) => true,
                (NoRefChoice::Yn(_), Choose::Yn) => true,
                _ => false,
            },
            _ => false,
        })
    }

    pub fn step(&mut self, action: Action) -> InnResult<GameState> {
        if !self.is_available_action(&action) {
            return Err(InnovationError::InvalidAction);
        }
        self.with_mut(|fields| {
            fields.logger.borrow_mut().act(action.clone());
            let game = *fields.players_ref;
            let action = action.to_ref(game);
            match action {
                RefAction::Step(action) => match fields.state {
                    State::Main => {
                        let player = game.player_at(fields.turn.player_id());
                        match action {
                            RefStepAction::Draw => {
                                // This current executor finder appears here and in Players.execute
                                // because either he's executing something or not.
                                game.draw(player, player.age())
                                    .map_err(|e| e.or_set_current_player(player.id()))?;
                                fields.turn.next_action()?;
                            }
                            RefStepAction::Meld(card) => {
                                game.meld(player, card)?;
                                fields.turn.next_action()?;
                            }
                            RefStepAction::Achieve(age) => {
                                game.try_achieve(player, &SingleAchievementView::Normal(age)).expect("Have checked action, corresponding achievement should be available.");
                                fields.turn.next_action()?;
                            }
                            RefStepAction::Execute(card) => {
                                *fields.state = State::Executing(game.execute(player, card));
                            }
                        }
                    }
                    State::Executing(_) => {
                        panic!("State and action mismatched");
                    }
                },
                RefAction::Executing(action) => match fields.state {
                    State::Main => panic!("State and action mismatched"),
                    State::Executing(state) => {
                        state.set_para(action);
                    }
                },
            }
            Ok(())
        })?;
        self.resume_execution()
    }

    fn resume_execution(&mut self) -> InnResult<GameState> {
        // helper enums/functions
        enum Info<'a> {
            Normal(ObsType<'a>),
            End(Vec<PlayerId>),
        }
        fn ok_normal(player: PlayerId, obs_type: ObsType) -> InnResult<(PlayerId, Info)> {
            Ok((player, Info::Normal(obs_type)))
        }

        match self.with_mut(|fields| {
            // resume execution, change to Main if ended,
            // and get current player and the obsType, which contains
            // some information if it is executing
            if let State::Executing(state) = fields.state {
                match state.resume() {
                    Some(Ok(st)) => {
                        let (p, o) = st.to_obs();
                        let id = p.id();
                        ok_normal(id, ObsType::Executing(o))
                    }
                    Some(Err(e)) => {
                        if let InnovationError::Win {
                            current_player,
                            situation,
                        } = e
                        {
                            Ok((
                                current_player.unwrap(),
                                Info::End(situation.winners(fields.players_ref)),
                            ))
                        } else {
                            Err(e)
                        }
                    }
                    None => {
                        *fields.state = State::Main;
                        fields.turn.next_action()?;
                        ok_normal(fields.turn.player_id(), ObsType::Main)
                    }
                }
            } else {
                ok_normal(fields.turn.player_id(), ObsType::Main)
            }
        })? {
            (player, Info::Normal(obs_type)) => {
                self.with_next_action_type_mut(|field| *field = obs_type.clone());
                Ok(GameState::Normal(self.observe(player, obs_type)))
            }
            (player, Info::End(winners)) => Ok(GameState::End(self.observe_end(player, winners))),
        }
    }

    fn observe<'a>(&'a self, id: usize, obs_type: ObsType<'a>) -> Observation {
        let players = *self.borrow_players_ref();
        Observation {
            acting_player: id,
            main_player: players.player_at(id).self_view(),
            other_players: players
                .players_from(id)
                .skip(1)
                .map(|p| p.other_view())
                .collect(),
            main_pile: players.main_card_pile.borrow().view(),
            turn: self.borrow_turn().turn(),
            obstype: obs_type,
        }
    }

    fn observe_end(&self, current_player: PlayerId, winners: Vec<PlayerId>) -> EndObservation {
        let players = *self.borrow_players_ref();
        EndObservation {
            players_from_current: players
                .ids_from(current_player)
                .map(|id| players.player_at(id).self_view())
                .collect(),
            main_pile: players.main_card_pile().borrow().view(),
            turn: self.borrow_turn().turn(),
            winners,
        }
    }

    pub fn history(&self) -> Vec<Game<'c>> {
        self.borrow_logger().borrow().history().to_vec()
    }

    pub fn current_game(&self) -> Option<Game<'c>> {
        self.borrow_logger().borrow().current_game().cloned()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        action::NoRefChoice, containers::VecSet, default_cards, state::ExecutionObs,
        utils::vec_eq_unordered,
    };

    #[test]
    fn create_game_player() {
        // will be used as achievement
        let pottery = default_cards::pottery();
        let archery = default_cards::archery();
        let code_of_laws = default_cards::code_of_laws();
        let agriculture = default_cards::agriculture();
        // will be used as achievement
        let monotheism = default_cards::monotheism();
        let philosophy = default_cards::philosophy();
        let cards = vec![
            &pottery,
            &archery,
            &code_of_laws,
            &agriculture,
            &monotheism,
            &philosophy,
        ];
        let mut game = OuterGame::init::<VecSet<&Card>>(2, cards);
        // do not call start(), in order to reduce cards used
        // card pile: 1[archery, code of laws, agriculture], 2[philosophy]
        game.step(Action::Step(NoRefStepAction::Draw))
            .expect("Action should be valid");
        // card pile: 1[code of laws, agriculture], 2[philosophy];
        // p1.hand[archery]; act1.cur.p2
        game.step(Action::Step(NoRefStepAction::Draw))
            .expect("Action should be valid");
        // card pile: 1[agriculture], 2[philosophy]; p1.hand[archery]; act2.cur.p2.hand[code of laws]
        println!("{:#?}", game.step(Action::Step(NoRefStepAction::Draw)));
        // card pile: 1[], 2[philosophy];
        // act1.cur.p1.hand[archery]; p2.hand[code of laws, agriculture]
        println!(
            "{:#?}",
            game.step(Action::Step(NoRefStepAction::Meld(String::from("Archery"))))
        );
        // card pile: 1[], 2[philosophy];
        // act2.cur.p1.board[archery]; p2.hand[code of laws, agriculture]
        {
            let obs = game
                .step(Action::Step(NoRefStepAction::Execute(String::from(
                    "Archery",
                ))))
                .expect("Action should be valid");
            // p2 must draw a 1, then give a card to p1
            // card pile: 1[], 2[];
            // act2.p1.board[archery.exe]; cur.p2.hand[code of laws, philosophy, agriculture]
            assert!(matches!(
                obs.as_normal().unwrap().obstype,
                ObsType::Executing(_)
            ))
        }
        // choose to transfer Philosophy
        {
            let obs = game
                .step(Action::Executing(NoRefChoice::Card(vec![String::from(
                    "Philosophy",
                )])))
                .expect("Action should be valid")
                .as_normal()
                .unwrap();
            // card pile: 1[], 2[];
            // p1.hand[philosophy].board[archery]; act1.cur.p2.hand[code of laws, agriculture]
            println!("{:#?}", obs);
            assert_eq!(obs.turn.player_id(), 1);
            assert!(matches!(obs.obstype, ObsType::Main));
        }
    }

    #[test]
    fn executing() {
        let archery = default_cards::archery();
        let pottery = default_cards::pottery();
        let agriculture = default_cards::agriculture();
        let cards = vec![&archery, &pottery, &agriculture];
        let mut game = OuterGame::config(
            cards,
            MainCardPile::new(vec![&pottery], Vec::new()),
            vec![
                PlayerBuilder::new::<VecSet<&Card>>().board(vec![&archery]),
                PlayerBuilder::new::<VecSet<&Card>>().hand(vec![&agriculture]),
            ],
            TurnBuilder::new(),
        );
        {
            // p1 executes 'Archery'.
            // p1 has 2 Castles, while p2 doesn't have any.
            // p1 demands p2 to draw a card 'Pottery' and transfer a 1 to p1's hand.
            let obs = game
                .step(Action::Step(NoRefStepAction::Execute("Archery".to_owned())))
                .expect("should be able to execute in this configured game")
                .as_normal()
                .expect("game should not end in this configured game");
            assert_eq!(obs.acting_player, 1);
            // TODO: maybe use HashSet in observation?
            assert!(vec_eq_unordered(
                &obs.main_player.hand,
                [&agriculture, &pottery]
            ));
            assert!(matches!(obs.obstype, ObsType::Executing(ExecutionObs {
                state: Choose::Card { min_num: 1, max_num: Some(1), from },
                card,
            }) if vec_eq_unordered(&from, [&agriculture, &pottery]) && card == &archery));
        }
        assert!(game.step(Action::Step(NoRefStepAction::Draw)).is_err());
        assert!(game.step(Action::Executing(NoRefChoice::Yn(true))).is_err());
        assert!(game
            .step(Action::Executing(NoRefChoice::Card(vec![])))
            .is_err());
        assert!(game
            .step(Action::Executing(NoRefChoice::Card(vec![
                "Archery".to_owned()
            ])))
            .is_err());
        assert!(game
            .step(Action::Executing(NoRefChoice::Card(vec![
                "Agriculture".to_owned(),
                "Pottery".to_owned()
            ])))
            .is_err());
        {
            let obs = game
                .step(Action::Executing(NoRefChoice::Card(vec![
                    "Agriculture".to_owned()
                ])))
                .unwrap()
                .as_normal()
                .unwrap();
            assert_eq!(obs.acting_player, 1);
            assert!(vec_eq_unordered(&obs.main_player.hand, [&pottery]));
            assert!(matches!(obs.obstype, ObsType::Main));
        }
        println!("Log messages: {:?}", game.current_game().unwrap().items);
    }
}
