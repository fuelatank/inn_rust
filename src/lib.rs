pub mod action;
pub mod auto_achieve;
pub mod board;
pub mod card;
pub mod card_pile;
pub mod containers;
pub mod error;
pub mod game;
pub mod logger;
pub mod observation;
pub mod player;
pub mod state;
pub mod structure;
pub mod turn;
pub mod utils;
pub mod xx;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
