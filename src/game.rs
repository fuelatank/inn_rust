use std::cell::RefCell;
use std::rc::Rc;

use generator::Gn;
use ouroboros::self_referencing;
use serde::Serialize;

use crate::{
    action::{Action, NoRefStepAction, RefAction, RefStepAction},
    card::{Achievement, Card, Dogma},
    card_pile::MainCardPile,
    containers::{BoxAchievementSet, BoxCardSet, CardSet},
    enums::{Color, Splay},
    error::{InnResult, InnovationError},
    flow::FlowState,
    logger::{Logger, Operation},
    observation::{ObsType, Observation},
    player::Player,
    state::State,
    structure::{
        AddToGame, Board, Hand, MainCardPile as MainCardPile_, Place, RemoveFromGame, Score,
    },
};

pub type RcCell<T> = Rc<RefCell<T>>;
pub type PlayerId = usize;

pub struct Players<'c> {
    cards: Vec<&'c Card>,
    logger: RcCell<Logger<'c>>,
    main_card_pile: RcCell<MainCardPile<'c>>,
    players: Vec<Player<'c>>,
}

impl<'c> Players<'c> {
    pub fn empty(logger: RcCell<Logger<'c>>) -> Players<'c> {
        Players {
            cards: Vec::new(),
            logger,
            main_card_pile: Rc::new(RefCell::new(MainCardPile::empty())),
            players: vec![],
        }
    }

    pub fn new<C, A>(
        num_players: usize,
        cards: Vec<&'c Card>,
        logger: RcCell<Logger<'c>>,
    ) -> Players<'c>
    where
        C: CardSet<'c, Card> + Default + 'c,
        A: CardSet<'c, Achievement> + Default + 'c,
    {
        let pile = Rc::new(RefCell::new(MainCardPile::new(cards.clone())));
        // Should logger cards be initialized here, or in other methods?
        logger.borrow_mut().start(pile.borrow().contents());
        Players {
            cards,
            logger: Rc::clone(&logger),
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

    pub fn players_from(&self, main_player_id: PlayerId) -> impl Iterator<Item = &Player<'c>> {
        (0..self.players.len())
            .map(move |i| &self.players[(i + main_player_id) % self.players.len()])
    }

    fn _ids_from(&self, main_player_id: PlayerId) -> impl Iterator<Item = PlayerId> {
        let len = self.players.len();
        (0..len).map(move |i| (i + main_player_id) % len)
    }

    pub fn main_card_pile(&self) -> &RcCell<MainCardPile<'c>> {
        &self.main_card_pile
    }

    pub fn draw<'g>(&'g self, player: &'g Player<'c>, age: u8) -> Option<&'c Card> {
        // transfer(Rc::clone(&self.main_pile), &self.hand, &age)
        self.transfer(MainCardPile_, player.with_id(Hand), age, ())
            .ok()
    }

    pub fn draw_and_meld<'g>(&'g self, player: &'g Player<'c>, age: u8) -> Option<&'c Card> {
        // transfer(Rc::clone(&self.main_pile), &self.main_board, &age)
        self.transfer(MainCardPile_, player.with_id(Board), age, true)
            .ok()
    }

    pub fn draw_and_score<'g>(&'g self, player: &'g Player<'c>, age: u8) -> Option<&'c Card> {
        // transfer(Rc::clone(&self.main_pile), &self.score_pile, &age)
        self.transfer(MainCardPile_, player.with_id(Score), age, ())
            .ok()
    }

    pub fn draw_and_tuck<'g>(&'g self, player: &'g Player<'c>, age: u8) -> Option<&'c Card> {
        self.transfer(MainCardPile_, player.with_id(Board), age, false)
            .ok()
    }

    pub fn meld<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> Option<&'c Card> {
        // transfer(&self.hand, &self.main_board, card)
        self.transfer(player.with_id(Hand), player.with_id(Board), card, true)
            .ok()
    }

    pub fn score<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> Option<&'c Card> {
        // transfer(&self.hand, &self.score_pile, card)
        self.transfer(player.with_id(Hand), player.with_id(Score), card, ())
            .ok()
    }

    pub fn tuck<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> Option<&'c Card> {
        // transfer(&self.hand, &self.main_board, card)
        self.transfer(player.with_id(Hand), player.with_id(Board), card, false)
            .ok()
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

    pub fn r#return<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> Option<&'c Card> {
        // transfer(&self.hand, Rc::clone(&self.main_pile), card)
        self.transfer(player.with_id(Hand), MainCardPile_, card, ())
            .ok()
    }

    pub fn execute<'g>(&'g self, player: &'g Player<'c>, card: &'c Card) -> FlowState<'c, 'g> {
        Gn::new_scoped_local(move |mut s| {
            let _main_icon = card.main_icon();
            let id = player.id();
            for dogma in card.dogmas() {
                match dogma {
                    Dogma::Share(flow) => {
                        // should filter out ineligible players
                        for player in self.players_from(id) {
                            let mut gen = flow(player, self);

                            // s.yield_from(gen); but with or(card)
                            let mut state = gen.resume();
                            while let Some(st) = state {
                                let a = s.yield_(st.or(card)).expect("Generator got None");
                                gen.set_para(a);
                                state = gen.resume();
                            }
                        }
                    }
                    Dogma::Demand(flow) => {
                        // should filter out ineligible players
                        for player in self.players_from(id).skip(1) {
                            let mut gen = flow(self.player_at(id), player, self);
                            // s.yield_from(gen); but with or(card)
                            let mut state = gen.resume();
                            while let Some(st) = state {
                                let a = s.yield_(st.or(card)).expect("Generator got None");
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
        if let Some(card) = card {
            to.add_to(card, self, add_param);
            self.logger
                .borrow_mut()
                .operate(Operation::Transfer(from.into(), to.into(), card));
            Ok(card)
        } else {
            Err(InnovationError::CardNotFound)
        }
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
}

impl<'c> OuterGame<'c> {
    pub fn init<C, A>(num_players: usize, cards: Vec<&'c Card>) -> OuterGame<'c>
    where
        C: CardSet<'c, Card> + Default + 'c,
        A: CardSet<'c, Achievement> + Default + 'c,
    {
        let logger = Rc::new(RefCell::new(Logger::new()));
        OuterGameBuilder {
            players: Players::new::<C, A>(num_players, cards, Rc::clone(&logger)),
            players_ref_builder: |players| players,
            turn: Turn::new(num_players, 0),
            logger,
            state: State::Main,
        }
        .build()
    }

    fn _is_available_step_action(&self, action: &NoRefStepAction) -> bool {
        self.with(|fields| match action {
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
        })
    }

    pub fn step(&mut self, action: Action) -> Observation {
        let (player, obs_type) = self.with_mut(|fields| {
            fields.logger.borrow_mut().act(action.clone());
            let game = *fields.players_ref;
            let action = action.to_ref(game);
            match action {
                RefAction::Step(action) => match fields.state {
                    State::Main => {
                        let player = game.player_at(fields.turn.player_id());
                        match action {
                            RefStepAction::Draw => {
                                game.draw(player, player.age());
                                fields.turn.next();
                            }
                            RefStepAction::Meld(card) => {
                                game.meld(player, card);
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
                    Some(st) => {
                        let (p, o) = st.to_obs();
                        let id = p.id();
                        (id, ObsType::Executing(o))
                    }
                    None => {
                        *fields.state = State::Main;
                        fields.turn.next();
                        (fields.turn.player_id(), ObsType::Main)
                    }
                }
            } else {
                (fields.turn.player_id(), ObsType::Main)
            }
        });
        self.observe(player, obs_type)
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
        let optics = Card::new(
            String::from("Optics"),
            3,
            Red,
            [Crown, Crown, Crown, Empty],
            dogma_fn::optics(),
            String::from("this is the doc of the card 'optics'"),
        );
        let cards = vec![&archery, &code_of_laws, &optics];
        let mut game = OuterGame::init::<VecSet<Card>, VecSet<Achievement>>(2, cards);
        game.step(Action::Step(NoRefStepAction::Draw));
        game.step(Action::Step(NoRefStepAction::Draw));
        println!("{:#?}", game.step(Action::Step(NoRefStepAction::Draw)));
        println!(
            "{:#?}",
            game.step(Action::Step(NoRefStepAction::Meld(String::from("Archery"))))
        );
        {
            let obs = game.step(Action::Step(NoRefStepAction::Execute(String::from(
                "Archery",
            ))));
            assert!(matches!(obs.obstype, ObsType::Executing(_)))
        }
        {
            let obs = game.step(Action::Executing(NoRefChoice::Card(vec![String::from(
                "Optics",
            )])));
            println!("{:#?}", obs);
            assert_eq!(obs.turn.player_id(), 1);
            assert!(matches!(obs.obstype, ObsType::Main));
        }
    }
}
