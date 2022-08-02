use crate::action::RefChoice;
use crate::card::Card;
use crate::containers::CardSet;
use crate::game::Players;
use crate::player::Player;
use crate::state::ExecutionState;
use generator::LocalGenerator;

pub type FlowState<'c, 'g> = LocalGenerator<'g, RefChoice<'c, 'g>, ExecutionState<'c, 'g>>;

pub type ShareFlow = for<'c, 'g> fn(&'g Player<'c>, &'g Players<'c>) -> FlowState<'c, 'g>;
pub type DemandFlow =
    for<'c, 'g> fn(&'g Player<'c>, &'g Player<'c>, &'g Players<'c>) -> FlowState<'c, 'g>;

#[cfg(test)]
mod tests {
    //use crate::game::transfer_selem;
    use super::*;
    use crate::card::Achievement;
    use crate::containers::Addable;

    fn _chemistry2<'a, T: CardSet<'a, Card>, U: Addable<'a, Achievement> + Default>(
    ) -> Box<dyn Fn(&mut Players, usize)> {
        // Player is inside Game
        // One player must be placed inside one game
        // Player is created when that Game is created
        Box::new(|_game, _player| {
            //game.draw_and_score(player, player.age() + 1);
        })
    }
    
    #[test]
    fn name() {
        // let mut game: Players = Players::empty();
        /*game.add_player(
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
            Box::new(VecSet::default()),
        );*/
        //let the_wheel = vec![];
        //let chemistry1 = vec![];
        //let optics = vec![];
    }
}
