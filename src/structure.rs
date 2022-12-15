use crate::{
    board::Board as Board_,
    card::Card,
    containers::{Addable, Removeable},
    enums::Color,
    error::{InnResult, InnovationError},
    game::Players,
    player::Player,
};

trait RemoveFromPlayer<'c, P> {
    fn remove_from(&self, player: &Player<'c>, param: P) -> Option<&'c Card>;
}

trait AddToPlayer<'c, P> {
    fn add_to(&self, card: &'c Card, player: &Player<'c>, param: P);
}

pub trait RemoveFromGame<'c, P> {
    fn remove_from(&self, game: &Players<'c>, param: P) -> Option<&'c Card>;
}

pub trait AddToGame<'c, P> {
    fn add_to(&self, card: &'c Card, game: &Players<'c>, param: P);
}

impl<'c, T, P> RemoveFromGame<'c, P> for (usize, T)
where
    T: RemoveFromPlayer<'c, P>,
{
    fn remove_from(&self, game: &Players<'c>, param: P) -> Option<&'c Card> {
        self.1.remove_from(game.player_at(self.0), param)
    }
}

impl<'c, T, P> AddToGame<'c, P> for (usize, T)
where
    T: AddToPlayer<'c, P>,
{
    fn add_to(&self, card: &'c Card, game: &Players<'c>, param: P) {
        self.1.add_to(card, game.player_at(self.0), param)
    }
}

pub struct Hand;

impl<'c, 'a> RemoveFromPlayer<'c, &'a Card> for Hand {
    fn remove_from(&self, player: &Player<'c>, param: &'a Card) -> Option<&'c Card> {
        player.hand.borrow_mut().remove(param)
    }
}

impl<'c> AddToPlayer<'c, ()> for Hand {
    fn add_to(&self, card: &'c Card, player: &Player<'c>, _param: ()) {
        player.hand.borrow_mut().add(card)
    }
}

pub struct Score;

impl<'c, 'a> RemoveFromPlayer<'c, &'a Card> for Score {
    fn remove_from(&self, player: &Player<'c>, param: &'a Card) -> Option<&'c Card> {
        player.score_pile.borrow_mut().remove(param)
    }
}

impl<'c> AddToPlayer<'c, ()> for Score {
    fn add_to(&self, card: &'c Card, player: &Player<'c>, _param: ()) {
        player.score_pile.borrow_mut().add(card)
    }
}

pub struct Board;

impl<'c, P> RemoveFromPlayer<'c, P> for Board
where
    Board_<'c>: Removeable<'c, Card, P>,
{
    fn remove_from(&self, player: &Player<'c>, param: P) -> Option<&'c Card> {
        <Board_ as Removeable<Card, P>>::remove(&mut *player.board().borrow_mut(), &param)
    }
}

impl<'c> AddToPlayer<'c, bool> for Board {
    fn add_to(&self, card: &'c Card, player: &Player<'c>, is_top: bool) {
        if is_top {
            player.board().borrow_mut().meld(card)
        } else {
            player.board().borrow_mut().tuck(card)
        }
    }
}

impl<'c> AddToPlayer<'c, usize> for Board {
    fn add_to(&self, card: &'c Card, player: &Player<'c>, index: usize) {
        player.board().borrow_mut().insert(card, index)
    }
}

pub struct MainCardPile;

impl<'c> RemoveFromGame<'c, u8> for MainCardPile {
    fn remove_from(&self, game: &Players<'c>, param: u8) -> Option<&'c Card> {
        game.main_card_pile().borrow_mut().remove(&param)
    }
}

impl<'c> AddToGame<'c, ()> for MainCardPile {
    fn add_to(&self, card: &'c Card, game: &Players<'c>, _param: ()) {
        game.main_card_pile().borrow_mut().add(card)
    }
}

#[derive(Copy, Clone)]
pub enum PlayerPlace {
    Hand,
    Score,
    Board,
}

impl From<Hand> for PlayerPlace {
    fn from(_: Hand) -> Self {
        PlayerPlace::Hand
    }
}

impl From<Score> for PlayerPlace {
    fn from(_: Score) -> Self {
        PlayerPlace::Score
    }
}

impl From<Board> for PlayerPlace {
    fn from(_: Board) -> Self {
        PlayerPlace::Board
    }
}

#[derive(Copy, Clone)]
pub enum Place {
    MainCardPile,
    Player(usize, PlayerPlace),
}

impl Place {
    pub fn hand(player: &Player) -> Place {
        Place::Player(player.id(), PlayerPlace::Hand)
    }

    pub fn score(player: &Player) -> Place {
        Place::Player(player.id(), PlayerPlace::Score)
    }

    pub fn board(player: &Player) -> Place {
        Place::Player(player.id(), PlayerPlace::Board)
    }
}

impl<T> From<(usize, T)> for Place
where
    T: Into<PlayerPlace>,
{
    fn from(t: (usize, T)) -> Self {
        Place::Player(t.0, t.1.into())
    }
}

impl From<MainCardPile> for Place {
    fn from(_: MainCardPile) -> Self {
        Place::MainCardPile
    }
}

pub enum RemoveParam<'c> {
    Age(u8),
    Card(&'c Card),
    Top(bool),
    ColoredTop(Color, bool),
    Index(usize),
    ColoredIndex(Color, usize),
    NoParam,
}

pub enum AddParam {
    Top(bool),
    Index(usize),
    NoParam,
}

impl<'c> RemoveParam<'c> {
    pub fn age(self) -> InnResult<u8> {
        if let RemoveParam::Age(age) = self {
            Ok(age)
        } else {
            Err(InnovationError::ParamUnwrapError)
        }
    }
}
