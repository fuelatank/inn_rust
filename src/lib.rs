pub mod board;
pub mod card;
pub mod card_pile;
pub mod containers;
pub mod enums;
pub mod flow;
pub mod game;
pub mod player;
pub mod xx;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
