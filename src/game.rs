use std::cell::RefCell;
use std::rc::Rc;

use generator::Gn;
use ouroboros::self_referencing;
use serde::Serialize;
use strum::IntoEnumIterator;

use crate::{
    action::{Action, NoRefChoice, NoRefStepAction, RefAction, RefStepAction},
    auto_achieve::AchievementManager,
    card::{Achievement, Card, Dogma, SpecialAchievement},
    card_pile::MainCardPile,
    containers::{BoxAchievementSet, BoxCardSet, CardSet},
    enums::{Color, Splay},
    error::{InnResult, InnovationError, WinningSituation},
    flow::FlowState,
    logger::{FnPureObserver, Logger, Operation, Subject},
    observation::{EndObservation, GameState, ObsType, Observation},
    player::Player,
    state::{Choose, State},
    structure::{
        AddToGame, Board, Hand, MainCardPile as MainCardPile_, Place, RemoveFromGame, Score,
    },
};

pub type RcCell<T> = Rc<RefCell<T>>;
pub type PlayerId = usize;

pub struct Players<'c> {
    cards: Vec<&'c Card>,
    logger: RefCell<Subject<'c>>,
    main_card_pile: RcCell<MainCardPile<'c>>,
    players: Vec<Player<'c>>,
}

impl<'c> Players<'c> {
    pub fn empty() -> Players<'c> {
        Players {
            cards: Vec::new(),
            logger: RefCell::new(Subject::new()),
            main_card_pile: Rc::new(RefCell::new(MainCardPile::empty())),
            players: vec![],
        }
    }

    pub fn new<C, A>(
        num_players: usize,
        cards: Vec<&'c Card>,
        logger: RcCell<Logger<'c>>,
        first_player: PlayerId,
    ) -> Players<'c>
    where
        C: CardSet<'c, Card> + Default + 'c,
        A: CardSet<'c, Achievement<'c>> + Default + 'c,
    {
        let pile = Rc::new(RefCell::new(MainCardPile::new(
            cards.clone(),
            SpecialAchievement::iter().collect(),
        )));
        let mut subject = Subject::new();
        subject.register_internal_owned(AchievementManager::new(
            SpecialAchievement::iter().collect(),
            first_player,
        ));
        // Should logger cards be initialized here, or in other methods?
        logger.borrow_mut().start(pile.borrow().contents());
        subject.register_external_owned(FnPureObserver::new(move |ev| {
            logger.borrow_mut().log(ev.clone())
        }));
        Players {
            cards,
            logger: RefCell::new(subject),
            main_card_pile: Rc::clone(&pile),
            players: (0..num_players)
                .map(|i| {
                    Player::new(
                        i,
                        Box::new(C::default()),
                        Box::new(C::default()),
                        Box::new(A::default()),
                    )
                })
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
        achievements: BoxAchievementSet<'c>,
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
    }

    pub fn draw_and_meld<'g>(&'g self, player: &'g Player<'c>, age: u8) -> InnResult<&'c Card> {
        // transfer(Rc::clone(&self.main_pile), &self.main_board, &age)
        self.transfer(MainCardPile_, player.with_id(Board), age, true)
    }

    pub fn draw_and_score<'g>(&'g self, player: &'g Player<'c>, age: u8) -> InnResult<&'c Card> {
        // transfer(Rc::clone(&self.main_pile), &self.score_pile, &age)
        self.transfer(MainCardPile_, player.with_id(Score), age, ())
    }

    pub fn draw_and_tuck<'g>(&'g self, player: &'g Player<'c>, age: u8) -> InnResult<&'c Card> {
        self.transfer(MainCardPile_, player.with_id(Board), age, false)
    }

    pub fn meld<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> InnResult<&'c Card> {
        // transfer(&self.hand, &self.main_board, card)
        self.transfer(player.with_id(Hand), player.with_id(Board), card, true)
    }

    pub fn score<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> InnResult<&'c Card> {
        // transfer(&self.hand, &self.score_pile, card)
        self.transfer(player.with_id(Hand), player.with_id(Score), card, ())
    }

    pub fn tuck<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> InnResult<&'c Card> {
        // transfer(&self.hand, &self.main_board, card)
        self.transfer(player.with_id(Hand), player.with_id(Board), card, false)
    }

    pub fn splay<'g>(&'g self, player: &'g Player<'c>, color: Color, direction: Splay) {
        player
            .board()
            .borrow_mut()
            .get_stack_mut(color)
            .splay(direction);
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
                            .filter_map(|(p, mask)| mask.then(|| p))
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
                            .filter_map(|(p, mask)| (!mask).then(|| p))
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
            generator::done!()
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
            // TODO: this does not allow observers to perform operations (log)
            self.logger
                .borrow_mut()
                .operate(Operation::Transfer(from.into(), to.into(), card), self);
            card
        })
    }

    pub fn transfer_card<Fr, To>(&self, from: Fr, to: To, card: &'c Card) -> InnResult<()>
    where
        Fr: RemoveFromGame<'c, &'c Card> + Into<Place>,
        To: AddToGame<'c, ()> + Into<Place>,
    {
        self.transfer(from, to, card, ()).map(|_| ())
    }
}

#[derive(Debug, Serialize)]
pub struct Turn {
    action: usize,
    num_players: usize,
    first_player: usize,
}

impl Turn {
    fn new(num_players: usize, first_player: usize) -> Turn {
        Turn {
            action: 0,
            num_players,
            first_player,
        }
    }

    pub fn first_player(&self) -> usize {
        self.first_player
    }

    pub fn is_second_action(&self) -> bool {
        self.action % 2 == 0
    }

    pub fn player_id(&self) -> usize {
        let a = (self.action + 1) / 2;
        (a + self.first_player) % self.num_players
    }

    fn next(&mut self) {
        self.action += 1;
    }
}

#[self_referencing]
pub struct OuterGame<'c> {
    players: Players<'c>,
    #[borrows(players)]
    players_ref: &'this Players<'c>,
    turn: Turn,
    logger: RcCell<Logger<'c>>,
    #[borrows()]
    #[covariant]
    state: State<'c, 'this>,
    next_action_type: ObsType<'c>,
}

impl<'c> OuterGame<'c> {
    pub fn init<C, A>(num_players: usize, cards: Vec<&'c Card>) -> OuterGame<'c>
    where
        C: CardSet<'c, Card> + Default + 'c,
        A: CardSet<'c, Achievement<'c>> + Default + 'c,
    {
        let logger = Rc::new(RefCell::new(Logger::new()));
        // TODO: structure not clear
        let turn = Turn::new(num_players, 0);
        OuterGameBuilder {
            players: Players::new::<C, A>(
                num_players,
                cards,
                Rc::clone(&logger),
                turn.first_player(),
            ),
            players_ref_builder: |players| players,
            turn,
            logger,
            state: State::Main,
            next_action_type: ObsType::Main,
        }
        .build()
    }

    fn is_available_action(&self, action: &Action) -> bool {
        self.with(|fields| match (action, fields.next_action_type) {
            (Action::Step(step), ObsType::Main) => match step {
                NoRefStepAction::Draw => true,
                NoRefStepAction::Meld(c) => {
                    let players = fields.players;
                    let player = &players.players[fields.turn.player_id()];
                    player.hand().as_vec().contains(&players.find_card(c))
                }
                NoRefStepAction::Achieve(_) => todo!(),
                NoRefStepAction::Execute(c) => {
                    let players = fields.players;
                    let player = &fields.players.players[fields.turn.player_id()];
                    player.board().borrow().contains(players.find_card(c))
                }
            },
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
        // helper enums/functions
        enum Info<'a> {
            Normal(ObsType<'a>),
            End(Vec<PlayerId>),
        }
        fn ok_normal(player: PlayerId, obs_type: ObsType) -> InnResult<(PlayerId, Info)> {
            Ok((player, Info::Normal(obs_type)))
        }
        match self.with_mut(|fields| {
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
                                fields.turn.next();
                            }
                            RefStepAction::Meld(card) => {
                                game.meld(player, card)?;
                                fields.turn.next();
                            }
                            RefStepAction::Achieve(_age) => {
                                fields.turn.next();
                                todo!()
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
                            let winners = match situation {
                                WinningSituation::SomeOne(p) => vec![p],
                                WinningSituation::ByScore => {
                                    let max_score = game
                                        .players_from(0)
                                        .map(|player| {
                                            // sort order
                                            (
                                                player.total_score(),
                                                player.achievements().as_vec().len(),
                                            )
                                        })
                                        .max()
                                        .unwrap();
                                    game.players_from(0)
                                        .filter_map(|player| {
                                            if (
                                                player.total_score(),
                                                player.achievements().as_vec().len(),
                                            ) == max_score
                                            {
                                                Some(player.id())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect()
                                }
                            };
                            Ok((current_player.unwrap(), Info::End(winners)))
                        } else {
                            Err(e)
                        }
                    }
                    None => {
                        *fields.state = State::Main;
                        fields.turn.next();
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
            main_player: players.player_at(id).self_view(),
            other_players: players
                .players_from(id)
                .skip(1)
                .map(|p| p.other_view())
                .collect(),
            main_pile: players.main_card_pile.borrow().view(),
            turn: self.borrow_turn(),
            obstype: obs_type,
        }
    }

    fn observe_end<'a>(
        &'a self,
        current_player: PlayerId,
        winners: Vec<PlayerId>,
    ) -> EndObservation {
        let players = *self.borrow_players_ref();
        EndObservation {
            players_from_current: players
                .ids_from(current_player)
                .map(|id| players.player_at(id).self_view())
                .collect(),
            main_pile: players.main_card_pile().borrow().view(),
            turn: self.borrow_turn(),
            winners,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        action::NoRefChoice,
        containers::VecSet,
        dogma_fn,
        enums::{Color::*, Icon::*},
    };

    #[test]
    fn turn() {
        let mut t1 = Turn::new(5, 1);
        assert_eq!(t1.player_id(), 1);
        assert_eq!(t1.is_second_action(), true);
        t1.next();
        assert_eq!(t1.player_id(), 2);
        assert_eq!(t1.is_second_action(), false);
        t1.next();
        assert_eq!(t1.player_id(), 2);
        assert_eq!(t1.is_second_action(), true);
        t1.next();
        assert_eq!(t1.player_id(), 3);
        assert_eq!(t1.is_second_action(), false);
        t1.next();
        assert_eq!(t1.player_id(), 3);
        assert_eq!(t1.is_second_action(), true);
        t1.next();
        assert_eq!(t1.player_id(), 4);
        assert_eq!(t1.is_second_action(), false);
        t1.next();
        assert_eq!(t1.player_id(), 4);
        assert_eq!(t1.is_second_action(), true);
        t1.next();
        assert_eq!(t1.player_id(), 0);
        assert_eq!(t1.is_second_action(), false);
        t1.next();
        assert_eq!(t1.player_id(), 0);
        assert_eq!(t1.is_second_action(), true);
        t1.next();
        assert_eq!(t1.player_id(), 1);
        assert_eq!(t1.is_second_action(), false);
        t1.next();
        assert_eq!(t1.player_id(), 1);
        assert_eq!(t1.is_second_action(), true);
        t1.next();
        t1.next();
        t1.next();
        t1.next();
        assert_eq!(t1.player_id(), 3);
        assert_eq!(t1.is_second_action(), true);
    }

    #[test]
    fn create_game_player() {
        // will be used as achievement
        let pottery = Card::new(
            "Pottery".to_owned(),
            1,
            Blue,
            [Empty, Leaf, Leaf, Leaf],
            dogma_fn::pottery(),
            "You may return up to three cards from your hand. If you returned any cards, draw and score a card of value equal to the number of cards you returned.".to_owned()
        );
        let archery = Card::new(
            String::from("Archery"),
            1,
            Red,
            [Castle, Lightblub, Empty, Castle],
            dogma_fn::archery(),
            String::from(""),
        );
        let code_of_laws = Card::new(
            String::from("Code of Laws"),
            1,
            Purple,
            [Empty, Crown, Crown, Leaf],
            dogma_fn::code_of_laws(),
            String::from("this is the doc of the card 'code of laws'"),
        );
        // will be used as achievement
        let monotheism = Card::new(
            "Monotheism".to_owned(),
            2,
            Purple,
            [Empty, Castle, Castle, Castle],
            dogma_fn::monotheism(),
            "I demand you transfer a top card on your board of a different color from any card on my board to my score pile! If you do, draw and tuck a 1!\nDraw and tuck a 1.".to_owned(),
        );
        let philosophy = Card::new(
            "Philosophy".to_owned(),
            2,
            Purple,
            [Empty, Lightblub, Lightblub, Lightblub],
            dogma_fn::philosophy(),
            "You may splay left any one color of your cards.\nYou may score a card from your hand."
                .to_owned(),
        );
        let cards = vec![&pottery, &archery, &code_of_laws, &monotheism, &philosophy];
        let mut game = OuterGame::init::<VecSet<Card>, VecSet<Achievement>>(2, cards);
        game.step(Action::Step(NoRefStepAction::Draw))
            .expect("Action should be valid");
        game.step(Action::Step(NoRefStepAction::Draw))
            .expect("Action should be valid");
        println!("{:#?}", game.step(Action::Step(NoRefStepAction::Draw)));
        println!(
            "{:#?}",
            game.step(Action::Step(NoRefStepAction::Meld(String::from("Archery"))))
        );
        {
            let obs = game
                .step(Action::Step(NoRefStepAction::Execute(String::from(
                    "Archery",
                ))))
                .expect("Action should be valid");
            assert!(matches!(
                obs.as_normal().unwrap().obstype,
                ObsType::Executing(_)
            ))
        }
        {
            let obs = game
                .step(Action::Executing(NoRefChoice::Card(vec![String::from(
                    "Philosophy",
                )])))
                .expect("Action should be valid");
            println!("{:#?}", obs);
            assert_eq!(obs.as_normal().unwrap().turn.player_id(), 1);
            assert!(matches!(obs.as_normal().unwrap().obstype, ObsType::Main));
        }
    }
}
