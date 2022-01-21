
use crate::enums::Color;
use crate::enums::Splay;

pub enum Place {
    Hand,
    ScorePile,
    Stack(Color),
    Board
}

pub enum CrossPlayer {
    Me,
    You
}

pub enum Age {
    Number(u8),
    Highest()
}

pub enum FlowElem {
    Draw(u8),
    DrawAndMeld(u8),
    DrawAndTuck(u8),
    DrawAndScore(u8),
    Splay(Splay),
    MayElse(Flow, Flow),
}

type Flow = Vec<FlowElem>;

mod tests {
    use crate::containers::Player;
use crate::card::Achievement;
    use crate::containers::CardSet;
    use crate::card::Card;
    use crate::containers::Game;
    use super::*;
    #[test]
    fn name() {
        let the_wheel = vec![
            FlowElem::Draw(1),
            FlowElem::Draw(1)
        ];
        let chemistry1 = vec![
            FlowElem::MayElse(vec![FlowElem::Splay(Splay::Right)], vec![])
        ];
        fn f<T: CardSet<Card>, U: CardSet<Achievement>>() -> Box<dyn Fn(Game<T, U>, Player<T, U>)> {
            // Player is inside Game
            // One player must be placed inside one game
            // Player is created when that Game is created
            Box::new(
                |game: Game<T, U>, player: Player<T, U>| {
                    player.draw(game.main_card_pile, player.age() + 1)
                }
            )
        }
        let chemistry2 = || {};
    }
}