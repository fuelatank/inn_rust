use crate::action::{Action, MainAction};
use crate::card::{Achievement, Card};
use crate::card_pile::MainCardPile;
use crate::containers::{BoxAchievementSet, BoxCardSet, CardSet};
use crate::observation::{ObsType, Observation, TurnView};
use crate::player::Player;
use crate::state::State;
use ouroboros::self_referencing;
use std::cell::RefCell;
use std::rc::Rc;

pub type RcCell<T> = Rc<RefCell<T>>;

pub struct Players<'c> {
    main_card_pile: RcCell<MainCardPile<'c>>,
    players: Vec<Player<'c>>,
}

impl<'c> Players<'c> {
    pub fn empty() -> Players<'c> {
        Players {
            main_card_pile: Rc::new(RefCell::new(MainCardPile::empty())),
            players: vec![],
        }
    }

    pub fn new<C, A>(num_players: usize, cards: Vec<&'c Card>) -> Players<'c>
    where
        C: CardSet<'c, Card> + Default + 'c,
        A: CardSet<'c, Achievement> + Default + 'c,
    {
        let pile = Rc::new(RefCell::new(MainCardPile::new(cards)));
        Players {
            main_card_pile: Rc::clone(&pile),
            players: (0..num_players)
                .map(|i| {
                    Player::new(
                        i,
                        Rc::clone(&pile),
                        Box::new(C::default()),
                        Box::new(C::default()),
                        Box::new(A::default()),
                    )
                })
                .collect(),
        }
    }

    pub fn add_player(
        &mut self,
        hand: BoxCardSet<'c>,
        score_pile: BoxCardSet<'c>,
        achievements: BoxAchievementSet<'c>,
    ) {
        let id = self.players.len();
        self.players.push(Player::new(
            id,
            Rc::clone(&self.main_card_pile),
            hand,
            score_pile,
            achievements,
        ))
    }

    pub fn num_players(&self) -> usize {
        self.players.len()
    }

    pub fn players(&self) -> Vec<&Player<'c>> {
        self.players.iter().collect()
    }

    pub fn player_at(&self, id: usize) -> &Player<'c> {
        &self.players[id]
    }

    pub fn players_from(&self, main_player_id: usize) -> impl Iterator<Item = &Player<'c>> {
        (0..self.players.len())
            .map(move |i| &self.players[(i + main_player_id) % self.players.len()])
    }
}

pub struct Turn {
    action: usize,
    num_players: usize,
    first_player: usize,
}

impl Turn {
    fn new(num_players: usize, first_player: usize) -> Turn {
        Turn {
            action: 0,
            num_players: num_players,
            first_player: first_player,
        }
    }

    fn is_second_action(&self) -> bool {
        self.action % 2 == 0
    }

    fn player_id(&self) -> usize {
        let a = (self.action + 1) / 2;
        (a + self.first_player) % self.num_players
    }

    fn next(&mut self) {
        self.action += 1;
    }
}

#[self_referencing]
struct OuterGame<'c> {
    players: Players<'c>,
    #[borrows(players)]
    players_ref: &'this Players<'c>,
    turn: Turn,
    #[borrows()]
    #[covariant]
    state: State<'c, 'this>,
}

impl<'c> OuterGame<'c> {
    fn init<C, A>(num_players: usize, cards: Vec<&'c Card>) -> OuterGame<'c>
    where
        C: CardSet<'c, Card> + Default + 'c,
        A: CardSet<'c, Achievement> + Default + 'c,
    {
        OuterGameBuilder {
            players: Players::new::<C, A>(num_players, cards),
            players_ref_builder: |players| &players,
            turn: Turn::new(num_players, 0),
            state: State::Main,
        }
        .build()
    }

    fn is_available_step_action(&self, action: &MainAction<'c>) -> bool {
        self.with(|fields| match action {
            MainAction::Draw => true,
            MainAction::Meld(c) => {
                let player = &fields.players.players[fields.turn.player_id()];
                player.hand.borrow().as_vec().contains(c)
            }
            MainAction::Achieve(_) => todo!(),
            MainAction::Execute(c) => {
                let player = &fields.players.players[fields.turn.player_id()];
                player.board().borrow().contains(c)
            }
        })
    }

    pub fn step(&mut self, action: Action<'c>) -> Observation {
        let (player, obs_type) = self.with_mut(|fields| {
            match action {
                Action::Step(action) => match fields.state {
                    State::Main => {
                        let player = (*fields.players_ref).player_at(fields.turn.player_id());
                        match action {
                            MainAction::Draw => {
                                player.draw(player.age());
                                fields.turn.next();
                            }
                            MainAction::Meld(card) => {
                                player.meld(card);
                                fields.turn.next();
                            }
                            MainAction::Achieve(age) => {
                                fields.turn.next();
                                todo!()
                            }
                            MainAction::Execute(card) => {
                                *fields.state =
                                    State::Executing(player.execute(card, *fields.players_ref));
                            }
                        }
                    }
                    State::Executing(_) => {
                        panic!("State and action mismatched");
                    }
                },
                Action::Executing(action) => match fields.state {
                    State::Main => panic!("State and action mismatched"),
                    State::Executing(state) => {
                        state.set_para(action.to_ref(*fields.players_ref));
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

mod tests {
    use super::*;
    use crate::{
        card::Dogma,
        containers::VecSet,
        dogma_fn::{archery, code_of_laws, optics},
        enums::{Color, Icon},
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
        let cards = vec![
            Card::new(
                String::from("Archery"),
                1,
                Color::Red,
                [Icon::Castle, Icon::Lightblub, Icon::Empty, Icon::Castle],
                vec![Dogma::Demand(archery)],
                String::from(""),
            ),
            Card::new(
                String::from("Code of Laws"),
                1,
                Color::Purple,
                [Icon::Empty, Icon::Crown, Icon::Crown, Icon::Leaf],
                vec![Dogma::Share(code_of_laws)],
                String::from("this is the doc of the card 'code of laws'"),
            ),
            Card::new(
                String::from("Optics"),
                3,
                Color::Red,
                [Icon::Crown, Icon::Crown, Icon::Crown, Icon::Empty],
                vec![Dogma::Share(optics)],
                String::from("this is the doc of the card 'optics'"),
            ),
        ];
        let mut game =
            OuterGame::init::<VecSet<Card>, VecSet<Achievement>>(2, cards.iter().collect());
        game.step(Action::Step(MainAction::Draw));
    }
}
