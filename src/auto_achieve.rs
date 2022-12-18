use strum::IntoEnumIterator;

use crate::{
    card::SpecialAchievement,
    enums::{Color, Icon, Splay},
    game::{PlayerId, Players},
    logger::{InternalObserver, Item, Operation, SimpleOp},
    player::Player,
    structure::{Place, PlayerPlace},
};

pub struct AchievementManager<'c> {
    available_achievements: Vec<(SpecialAchievement, Box<dyn Achievement<'c>>)>,
    acting_player: PlayerId, // may be a duplicated Turn?
}

impl<'c> AchievementManager<'c> {
    pub fn new(first_player: PlayerId) -> Self {
        Self {
            available_achievements: vec![
                (
                    SpecialAchievement::Monument,
                    Box::new(Monument::new(first_player)),
                ),
                (SpecialAchievement::Empire, Box::new(Empire)),
                (SpecialAchievement::World, Box::new(World)),
                (SpecialAchievement::Wonder, Box::new(Wonder)),
                (SpecialAchievement::Universe, Box::new(Universe)),
            ],
            acting_player: first_player,
        }
    }
}

impl<'c> InternalObserver<'c> for AchievementManager<'c> {
    fn on_notify(&mut self, event: &Item<'c>, game: &Players<'c>) {
        // TODO: who is the "current player" that gets the achievement if two players
        // satisty the condition at exactly the same time?
        if let Item::ChangeTurn(_prev, next) = event {
            self.acting_player = *next;
        }
        for (_card, check) in self.available_achievements.iter_mut() {
            let interesting_players = check.update_interested(event);
            let order = game.ids_from(self.acting_player);
            for player in order
                .filter(|id| interesting_players.contains(id))
                .map(|id| game.player_at(id))
            {
                if check.further_check(game, player) {
                    // TODO: achieve
                    break;
                }
            }
        }
    }
}

trait Achievement<'c> {
    fn update_interested(&mut self, event: &Item<'c>) -> Vec<PlayerId>;
    fn further_check(&self, game: &Players<'c>, player: &Player<'c>) -> bool;
}

fn check_board<'c>(event: &Item<'c>) -> Vec<PlayerId> {
    if let Item::Operation(Operation::Transfer(from, to, _card)) = event {
        let mut res = Vec::new();
        if let Place::Player(from_player, PlayerPlace::Board) = from {
            res.push(*from_player);
        }
        if let Place::Player(to_player, PlayerPlace::Board) = to {
            if !res.contains(to_player) {
                res.push(*to_player);
            }
        }
        res
    } else {
        Vec::new()
    }
}
struct Monument {
    scored: usize,
    tucked: usize,
    current_player: PlayerId,
}

impl Monument {
    fn new(first_player: PlayerId) -> Self {
        Self {
            scored: 0,
            tucked: 0,
            current_player: first_player,
        }
    }
}

impl<'c> Achievement<'c> for Monument {
    fn update_interested(&mut self, event: &Item<'c>) -> Vec<PlayerId> {
        match event {
            Item::Operation(Operation::SimpleOp(SimpleOp::Score, player, _))
                if *player == self.current_player =>
            {
                self.scored += 1;
                if self.scored == 6 {
                    return vec![*player];
                }
            }
            Item::Operation(Operation::SimpleOp(SimpleOp::Tuck, player, _))
                if *player == self.current_player =>
            {
                self.tucked += 1;
                if self.tucked == 6 {
                    return vec![*player];
                }
            }
            Item::ChangeTurn(_prev, next) => {
                self.scored = 0;
                self.tucked = 0;
                self.current_player = *next;
            }
            _ => {}
        }
        return Vec::new();
    }

    fn further_check(&self, _game: &Players<'c>, _player: &Player<'c>) -> bool {
        true
    }
}

struct Empire;

impl<'c> Achievement<'c> for Empire {
    fn update_interested(&mut self, event: &Item<'c>) -> Vec<PlayerId> {
        check_board(event)
    }

    fn further_check(&self, _game: &Players<'c>, player: &Player<'c>) -> bool {
        let mut icons = player.board().borrow().icon_count().into_map();
        icons.remove(&Icon::Empty);
        icons.into_iter().all(|(_, count)| count >= 3)
    }
}

struct World;

impl<'c> Achievement<'c> for World {
    fn update_interested(&mut self, event: &Item<'c>) -> Vec<PlayerId> {
        check_board(event)
    }

    fn further_check(&self, _game: &Players<'c>, player: &Player<'c>) -> bool {
        player.board().borrow().icon_count()[&Icon::Clock] >= 12
    }
}

struct Wonder;

impl<'c> Achievement<'c> for Wonder {
    fn update_interested(&mut self, event: &Item<'c>) -> Vec<PlayerId> {
        if let Item::Operation(Operation::Splay(player, _, direction)) = event {
            if *direction == Splay::Right || *direction == Splay::Up {
                return vec![*player];
            }
        }
        Vec::new()
    }

    fn further_check(&self, _game: &Players<'c>, player: &Player<'c>) -> bool {
        let board = player.board().borrow();
        Color::iter()
            .map(|color| board.get_stack(color))
            .all(|stack| stack.is_splayed(Splay::Right) || stack.is_splayed(Splay::Up))
    }
}

struct Universe;

impl<'c> Achievement<'c> for Universe {
    fn update_interested(&mut self, event: &Item<'c>) -> Vec<PlayerId> {
        check_board(event)
    }

    fn further_check(&self, _game: &Players<'c>, player: &Player<'c>) -> bool {
        let top_cards = player.board().borrow().top_cards();
        top_cards.len() == 5 && top_cards.into_iter().all(|card| card.age() >= 8)
    }
}
