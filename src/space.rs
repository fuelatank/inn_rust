use std::iter::{repeat, repeat_with};

use enum_as_inner::EnumAsInner;
use rand::Rng as _;
use rand_distr::{Distribution as _, Poisson, Uniform};
use strum::IntoEnumIterator as _;

use crate::{
    action::{Action, NoRefChoice, NoRefStep},
    board::Board,
    card::{Age, Card, Color, Splay},
    observation::{ObsType, Observation, SingleAchievementView},
    state::Choose,
};

#[derive(Clone)]
pub enum Space {
    Discrete(usize),
    Box(Option<f32>, Option<f32>, usize), // identical bound, 1 dim
    MultiBinary(usize),
    OneOf(Vec<Space>),
    Sequence(Box<Space>),
    Dict(Vec<(String, Space)>),
    Tuple(Vec<Space>),
}

impl Space {
    pub fn contains(&self, instance: &Instance) -> bool {
        match (self, instance) {
            (Space::Discrete(n), Instance::Discrete(i)) => i < n,
            (Space::Box(low, high, dim), Instance::Box(values)) => {
                values.len() == *dim
                    && values.iter().all(|v| {
                        low.is_none_or(|low| low <= *v) && high.is_none_or(|high| *v <= high)
                    })
            }
            (Space::MultiBinary(n), Instance::MultiBinary(values)) => values.len() == *n,
            (Space::OneOf(spaces), Instance::OneOf(index, instance)) => {
                *index < spaces.len() && spaces[*index].contains(instance)
            }
            (Space::Sequence(space), Instance::Sequence(instances)) => {
                instances.iter().all(|instance| space.contains(instance))
            }
            (Space::Dict(space_dict), Instance::Dict(instance_dict)) => {
                space_dict.len() == instance_dict.len()
                    && space_dict.iter().all(|(key, space)| {
                        instance_dict
                            .iter()
                            .find(|(k, _)| k == key)
                            .map_or(false, |(_, instance)| space.contains(instance))
                    })
            }
            (Space::Tuple(spaces), Instance::Tuple(instances)) => {
                spaces.len() == instances.len()
                    && spaces
                        .iter()
                        .zip(instances)
                        .all(|(space, instance)| space.contains(instance))
            }
            _ => false,
        }
    }

    pub fn sample(&self) -> Instance {
        let mut rng = rand::rng();
        match self {
            Space::Discrete(n) => Instance::Discrete(rng.random_range(0..*n)),
            Space::Box(Some(low), Some(high), dim) => Instance::Box(
                Uniform::new(low, high)
                    .unwrap()
                    .sample_iter(&mut rng)
                    .take(*dim)
                    .collect(),
            ),
            Space::Box(_, _, _) => unimplemented!("maybe add unbounded sample later"),
            Space::MultiBinary(n) => {
                Instance::MultiBinary(repeat_with(|| rng.random()).take(*n).collect())
            }
            Space::OneOf(spaces) => todo!(),
            Space::Sequence(space) => Instance::Sequence(
                repeat_with(|| space.sample())
                    .take(Poisson::new(1.).unwrap().sample(&mut rng) as usize)
                    .collect(),
            ),
            Space::Dict(items) => todo!(),
            Space::Tuple(spaces) => todo!(),
        }
    }
}

#[derive(EnumAsInner)]
pub enum Instance {
    Discrete(usize),
    Box(Vec<f32>),
    MultiBinary(Vec<bool>),
    OneOf(usize, Box<Instance>),
    Sequence(Vec<Instance>),
    Dict(Vec<(String, Instance)>),
    Tuple(Vec<Instance>),
}

impl Instance {
    fn multi_binary_from_set<T>(
        items: impl Iterator<Item = T>,
        index: &impl Fn(T) -> usize,
        len: usize,
        default_value: bool,
    ) -> Instance {
        let mut v: Vec<bool> = repeat(default_value).take(len).collect();
        for item in items {
            v[index(item)] = !default_value;
        }
        Instance::MultiBinary(v)
    }
}

pub fn action_space() -> Space {
    Space::OneOf(vec![
        Space::Discrete(1),                              // draw
        Space::Discrete(105),                            // meld
        Space::Discrete(9),                              // achieve
        Space::Discrete(5),                              // execute
        Space::Discrete(2),                              // yn
        Space::Discrete(1),                              // opponent
        Space::Sequence(Box::new(Space::Discrete(105))), // card(s)
    ])
}

pub fn observation_space_2p() -> Space {
    Space::Dict(vec![
        ("action".to_owned(), Space::Discrete(4)), // two players * two actions
        ("executing".to_owned(), Space::Discrete(106)),
        ("choose".to_owned(), Space::Discrete(4)),
        ("card_pile".to_owned(), Space::Box(Some(0.), Some(14.), 10)),
        ("available_achievements".to_owned(), Space::MultiBinary(14)),
        ("self_hand".to_owned(), Space::MultiBinary(105)),
        ("self_score".to_owned(), Space::MultiBinary(105)),
        (
            "self_board".to_owned(),
            Space::Tuple(
                repeat(Space::Sequence(Box::new(Space::Discrete(105))))
                    .take(5)
                    .collect(),
            ),
        ),
        (
            "self_board_splay".to_owned(),
            Space::Tuple(repeat(Space::Discrete(4)).take(5).collect()),
        ),
        (
            "self_achievement".to_owned(),
            Space::Box(Some(0.), Some(6.), 1),
        ),
        ("other_hand".to_owned(), Space::Box(Some(0.), Some(14.), 10)),
        (
            "other_score".to_owned(),
            Space::Box(Some(0.), Some(14.), 10),
        ),
        (
            "other_board".to_owned(),
            Space::Tuple(
                repeat(Space::Sequence(Box::new(Space::Discrete(105))))
                    .take(5)
                    .collect(),
            ),
        ),
        (
            "other_board_splay".to_owned(),
            Space::Tuple(repeat(Space::Discrete(4)).take(5).collect()),
        ),
        (
            "other_achievement".to_owned(),
            Space::Box(Some(0.), Some(6.), 1),
        ),
    ])
}

fn map_board(board: &Board, card_index: impl Fn(&Card) -> usize) -> Instance {
    Instance::Tuple(
        Color::iter()
            .map(|color| {
                let stack = board.get_stack(color);
                Instance::Sequence(
                    stack
                        .cards
                        .iter()
                        .map(|card| Instance::Discrete(card_index(card)))
                        .collect(),
                )
            })
            .collect(),
    )
}

fn map_board_splay(board: &Board) -> Instance {
    Instance::Tuple(
        Color::iter()
            .map(|color| {
                let stack = board.get_stack(color);
                Instance::Discrete(match stack.splay {
                    Splay::NoSplay => 0,
                    Splay::Left => 1,
                    Splay::Right => 2,
                    Splay::Up => 3,
                })
            })
            .collect(),
    )
}

fn map_back_card_set(ages: &[Age]) -> Instance {
    let mut freq: Vec<_> = repeat(0).take(10).collect();
    for age in ages {
        freq[(age - 1) as usize] += 1;
    }
    Instance::Box(freq.into_iter().map(|n| n as f32).collect())
}

pub fn map_observation_2p(
    obs: &Observation,
    card_index: &impl Fn(&Card) -> usize,
    achievement_index: &impl Fn(&SingleAchievementView) -> usize,
) -> Option<Instance> {
    // assuming the acting player (who needs to make choice) is the main player (who is observing)
    if obs.other_players.len() != 1 {
        return None;
    }
    let relative_current_player = if obs.turn.current_player() == obs.acting_player {
        0
    } else {
        1
    };
    let step = if obs.turn.is_second_step() { 1 } else { 0 };
    Some(Instance::Dict(vec![
        (
            "action".to_owned(),
            Instance::Discrete(relative_current_player * 2 + step),
        ),
        (
            "executing".to_owned(),
            Instance::Discrete(match &obs.obstype {
                ObsType::Main => 105,
                ObsType::Executing(c) => card_index(&c.card),
            }),
        ),
        (
            "choose".to_owned(),
            Instance::Discrete(match &obs.obstype {
                ObsType::Main => 0,
                ObsType::Executing(c) => match &c.state {
                    Choose::Card { .. } => 1,
                    Choose::Opponent => 2,
                    Choose::Yn => 3,
                },
            }),
        ),
        (
            "card_pile".to_owned(),
            Instance::Box(obs.main_pile.into_iter().map(|n| n as f32).collect()),
        ),
        (
            "available_achievements".to_owned(),
            Instance::multi_binary_from_set(
                obs.main_player
                    .achievements
                    .iter()
                    .chain(obs.other_players[0].achievements.iter()),
                achievement_index,
                14,
                true,
            ),
        ),
        (
            "self_hand".to_owned(),
            Instance::multi_binary_from_set(
                obs.main_player.hand.clone().into_iter(),
                card_index,
                105,
                false,
            ),
        ),
        (
            "self_score".to_owned(),
            Instance::multi_binary_from_set(
                obs.main_player.score.clone().into_iter(),
                card_index,
                105,
                false,
            ),
        ),
        (
            "self_board".to_owned(),
            map_board(&obs.main_player.board, card_index),
        ),
        (
            "self_board_splay".to_owned(),
            map_board_splay(&obs.main_player.board),
        ),
        (
            "self_achievement".to_owned(),
            Instance::Discrete(obs.main_player.achievements.len()),
        ),
        (
            "other_hand".to_owned(),
            map_back_card_set(&obs.other_players[0].hand),
        ),
        (
            "other_score".to_owned(),
            map_back_card_set(&obs.other_players[0].score),
        ),
        (
            "other_board".to_owned(),
            map_board(&obs.other_players[0].board, card_index),
        ),
        (
            "other_board_splay".to_owned(),
            map_board_splay(&obs.other_players[0].board),
        ),
        (
            "other_achievement".to_owned(),
            Instance::Discrete(obs.other_players[0].achievements.len()),
        ),
    ]))
}

pub fn map_action(action: &Instance, obs: &Observation, card_names: &[String]) -> Option<Action> {
    let Instance::OneOf(index, action) = action else {
        return None;
    };
    Some(match index {
        0 => Action::Step(NoRefStep::Draw),
        1 => Action::Step(NoRefStep::Meld(card_names[*action.as_discrete()?].clone())),
        2 => Action::Step(NoRefStep::Achieve(*action.as_discrete()? as u8 + 1)),
        3 => Action::Step(NoRefStep::Execute(
            obs.main_player.board.top_cards()[*action.as_discrete()?]
                .name()
                .to_owned(),
        )),
        4 => Action::Executing(NoRefChoice::Yn(*action.as_discrete()? == 1)),
        5 => Action::Executing(NoRefChoice::Opponent((obs.acting_player + 1) % 2)),
        6 => Action::Executing(NoRefChoice::Card(
            action
                .as_sequence()?
                .iter()
                .map(Instance::as_discrete)
                .collect::<Option<Vec<_>>>()?
                .into_iter()
                .map(|i| card_names[*i].clone())
                .collect(),
        )),
        _ => return None,
    })
}
