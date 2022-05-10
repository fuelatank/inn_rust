
use crate::game::Player;
use crate::game::BoxCardSet;
use crate::containers::CardSet;
use crate::enums::Icon;
use crate::card::Card;

pub enum ExecutingState<'a> {
    Done,
    ChooseAnyCard {
        min_num: u8,
        max_num: Option<u8>,
        from: &'a [&'a Card],
        callback: Box<dyn Fn(Option<&'a Vec<&'a Card>>) -> ExecutingState<'a> + 'a>
    },
    ChooseAnOpponent {
        callback: Box<dyn Fn(&'a Player<'a>) -> ExecutingState<'a> + 'a>
    }
}

pub type Flow = Box<dyn for<'a> Fn(&'a Player) -> ExecutingState<'a>>;

mod tests {
    //use crate::game::transfer_elem;
    use crate::containers::VecSet;
    use crate::game::Game;
    use crate::containers::Addable;
    use crate::game::Player;
    use crate::card::Achievement;
    use super::*;

    fn chemistry2<'a, T: CardSet<'a, Card>, U: Addable<'a, Achievement> + Default>() -> Box<dyn Fn(&mut Game, usize)> {
        // Player is inside Game
        // One player must be placed inside one game
        // Player is created when that Game is created
        Box::new(
            |game, player| {
                //game.draw_and_score(player, player.age() + 1);
            }
        )
    }

    fn opticsxx() -> Box<dyn for<'a> Fn(&'a mut Game<'a>, usize) -> ExecutingState<'a>>
    {
        Box::new(
            |game, player_index| {
                let card = game.draw_and_meld(player_index, &3).unwrap();
                if card.contains(Icon::Crown) {
                    game.draw_and_score(player_index, &4);
                    return ExecutingState::Done;
                } else {
                    return ExecutingState::ChooseAnyCard {
                        min_num: 1,
                        max_num: Some(1),
                        from: game.player(player_index).score_pile.as_slice(),
                        callback: Box::new(move |cards: Option<&Vec<&Card>>| {
                            return ExecutingState::ChooseAnOpponent {
                                callback: Box::new(move |opponent: &Player| {
                                    //transfer_elem(&mut player.score_pile, &opponent.score_pile, &cards.unwrap()[0]);
                                    return ExecutingState::Done;
                                })
                            };
                        })
                    };
                }
            }
        )
    }

    #[test]
    fn name() {
        let game: Game = Game::new();
        game.add_player(Box::new(VecSet::default()), Box::new(VecSet::default()), Box::new(VecSet::default()));
        let the_wheel = vec![
        ];
        let chemistry1 = vec![
        ];
        let optics = vec![
        ];
    }
}