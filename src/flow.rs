
use crate::containers::CardSet;
use crate::enums::Icon;
use crate::card::Card;

enum ExecutingState<'a, T: CardSet<Card>> {
    Done,
    ChooseAnyCard {
        min_num: u8,
        max_num: Option<u8>,
        from: &'a T,
        callback: Callback<'a, T>
    },
    ChooseAnOpponent {
        callback: Callback<'a, T>
    }
}

type Callback<'a, T> = Box<dyn Fn(Option<&Vec<Card>>) -> ExecutingState<'a, T>>;
type Flow<'a, T> = Box<dyn Fn() -> ExecutingState<'a, T>>;

mod tests {
    use crate::game::transfer_elem;
    use crate::containers::VecSet;
    use crate::game::Game;
    use crate::containers::Addable;
    use crate::game::Player;
    use crate::card::Achievement;
    use super::*;

    fn chemistry2<T: CardSet<Card>, U: Addable<Achievement> + Default>() -> Box<dyn Fn(Player<T, U>)> {
        // Player is inside Game
        // One player must be placed inside one game
        // Player is created when that Game is created
        Box::new(
            |player| {
                player.draw_and_score(player.age() + 1);
            }
        )
    }

    fn opticsxx<T, U>() -> Box<dyn Fn(Player<T, U>) -> ExecutingState<'_, T>>
    where
        T: CardSet<Card>,
        U: Addable<Achievement> + Default
    {
        Box::new(
            |player| {
                let card = player.draw_and_meld(3);
                if card.contains(Icon::Crown) {
                    player.draw_and_score(4);
                    return ExecutingState::Done;
                } else {
                    return ExecutingState::ChooseAnyCard {
                        min_num: 1,
                        max_num: Some(1),
                        from: &player.score_pile,
                        callback: |cards| {
                        return ExecutingState::ChooseAnOpponent{
                            callback: |opponent| {
                                transfer_elem(&player.score_pile, opponent.score_pile(), cards[0])
                            }
                        }}
                    };
                }
            }
        )
    }

    #[test]
    fn name() {
        let game: Game<VecSet<Card>, VecSet<Achievement>> = Game::new();
        game.add_player();
        let the_wheel = vec![
        ];
        let chemistry1 = vec![
        ];
        let optics = vec![
        ];
    }
}