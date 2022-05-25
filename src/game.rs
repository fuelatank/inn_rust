use crate::action::{Action, StepAction};
use crate::card::{Achievement, Card};
use crate::card_pile::MainCardPile;
use crate::containers::{BoxAchievementSet, BoxCardSet, CardSet};
use crate::player::Player;
use crate::state::State;
use ouroboros::self_referencing;
use std::cell::RefCell;
use std::rc::Rc;

pub type RcCell<T> = Rc<RefCell<T>>;

pub struct InnerGame<'c> {
    main_card_pile: RcCell<MainCardPile<'c>>,
    players: Vec<Player<'c>>,
}

impl<'c> InnerGame<'c> {
    pub fn empty() -> InnerGame<'c> {
        InnerGame {
            main_card_pile: Rc::new(RefCell::new(MainCardPile::empty())),
            players: vec![],
        }
    }

    pub fn new<C, A>(num_players: usize, cards: Vec<&'c Card>) -> InnerGame<'c>
    where
        C: CardSet<'c, Card> + Default + 'c,
        A: CardSet<'c, Achievement> + Default + 'c,
    {
        let pile = Rc::new(RefCell::new(MainCardPile::new(cards)));
        InnerGame {
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

#[self_referencing]
struct OuterGame<'c> {
    players: InnerGame<'c>,
    #[borrows(players)]
    players_ref: &'this InnerGame<'c>,
    current_player_id: usize,
    is_second_action: bool,
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
            players: InnerGame::new::<C, A>(num_players, cards),
            players_ref_builder: |players| &players,
            current_player_id: 0,
            is_second_action: true,
            state: State::Main,
        }
        .build()
    }

    fn is_available_step_action(&self, action: &StepAction<'c>) -> bool {
        self.with(|fields| match action {
            StepAction::Draw => true,
            StepAction::Meld(c) => {
                let player = &fields.players.players[*fields.current_player_id];
                player.hand.borrow().as_vec().contains(c)
            }
            StepAction::Achieve(_) => todo!(),
            StepAction::Execute(c) => {
                let player = &fields.players.players[*fields.current_player_id];
                player.board().borrow().contains(c)
            }
        })
    }

    pub fn step(&mut self, action: Action<'c>) {
        self.with_mut(|fields| {
            match action {
                Action::Step(action) => match fields.state {
                    State::Main => {
                        let player = (*fields.players_ref).player_at(*fields.current_player_id);
                        match action {
                            StepAction::Draw => {
                                player.draw(player.age());
                            }
                            StepAction::Meld(card) => {
                                player.meld(card);
                            }
                            StepAction::Achieve(age) => {
                                todo!()
                            }
                            StepAction::Execute(card) => {
                                *fields.state =
                                    State::Executing(player.execute(card, *fields.players_ref));
                            }
                        }
                        if *fields.is_second_action {
                            *fields.current_player_id =
                                (*fields.current_player_id + 1) % fields.players_ref.num_players();
                        }
                        *fields.is_second_action = !*fields.is_second_action;
                    }
                    State::Executing(_) => {
                        panic!("State and action mismatched");
                    }
                },
                Action::Executing(action) => match fields.state {
                    State::Main => panic!("State and action mismatched"),
                    State::Executing(state) => {
                        state.set_para(action.take_player(*fields.players_ref));
                    }
                },
            }
            if let State::Executing(state) = fields.state {
                let _a = state.resume();
                if state.is_done() {
                    *fields.state = State::Main;
                }
            }
        })
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
        game.step(Action::Step(StepAction::Draw));
    }
}
