use crate::{
    board::Board as Board_,
    card::Card,
    containers::{Addable, Removeable},
    enums::Color,
    error::{InnResult, InnovationError, WinningSituation},
    game::Players,
    player::Player,
};

trait RemoveFromPlayer<'c, P> {
    fn remove_from(&self, player: &Player<'c>, param: P) -> InnResult<&'c Card>;
}

trait TestRemoveFromPlayer<'c, P>: RemoveFromPlayer<'c, P> {
    /// Checks if it can successfully find a card using the param,
    /// without actually removing it.
    ///
    /// Should return `Ok` if and only if `remove_from` returns `Ok`, and
    /// the `Err` returned should be the same as in `remove_from`.
    fn test_remove(&self, player: &Player<'c>, param: P) -> InnResult<()>;
}

trait AddToPlayer<'c, P> {
    fn add_to(&self, card: &'c Card, player: &Player<'c>, param: P);
}

pub trait RemoveFromGame<'c, P> {
    fn remove_from(&self, game: &Players<'c>, param: P) -> InnResult<&'c Card>;
}

pub trait TestRemoveFromGame<'c, P>: RemoveFromGame<'c, P> {
    /// Checks if it can successfully find a card using the param,
    /// without actually removing it.
    ///
    /// Should return `Ok` if and only if `remove_from` returns `Ok`, and
    /// the `Err` returned should be the same as in `remove_from`.
    fn test_remove(&self, game: &Players<'c>, param: P) -> InnResult<()>;
}

pub trait AddToGame<'c, P> {
    fn add_to(&self, card: &'c Card, game: &Players<'c>, param: P);
}

impl<'c, T, P> RemoveFromGame<'c, P> for (usize, T)
where
    T: RemoveFromPlayer<'c, P>,
{
    fn remove_from(&self, game: &Players<'c>, param: P) -> InnResult<&'c Card> {
        self.1.remove_from(game.player_at(self.0), param)
    }
}

impl<'c, T, P> TestRemoveFromGame<'c, P> for (usize, T)
where
    T: TestRemoveFromPlayer<'c, P>,
{
    fn test_remove(&self, game: &Players<'c>, param: P) -> InnResult<()> {
        self.1.test_remove(game.player_at(self.0), param)
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
    fn remove_from(&self, player: &Player<'c>, param: &'a Card) -> InnResult<&'c Card> {
        player
            .hand
            .borrow_mut()
            .remove(param)
            .ok_or(InnovationError::CardNotFound)
    }
}

impl<'c, 'a> TestRemoveFromPlayer<'c, &'a Card> for Hand {
    fn test_remove(&self, player: &Player<'c>, param: &'a Card) -> InnResult<()> {
        if player.hand().as_iter().any(|card| param == card) {
            Ok(())
        } else {
            Err(InnovationError::CardNotFound)
        }
    }
}

impl<'c> AddToPlayer<'c, ()> for Hand {
    fn add_to(&self, card: &'c Card, player: &Player<'c>, _param: ()) {
        player.hand.borrow_mut().add(card)
    }
}

pub struct Score;

impl<'c, 'a> RemoveFromPlayer<'c, &'a Card> for Score {
    fn remove_from(&self, player: &Player<'c>, param: &'a Card) -> InnResult<&'c Card> {
        player
            .score_pile
            .borrow_mut()
            .remove(param)
            .ok_or(InnovationError::CardNotFound)
    }
}

impl<'c, 'a> TestRemoveFromPlayer<'c, &'a Card> for Score {
    fn test_remove(&self, player: &Player<'c>, param: &'a Card) -> InnResult<()> {
        if player.score_pile().as_iter().any(|card| param == card) {
            Ok(())
        } else {
            Err(InnovationError::CardNotFound)
        }
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
    Board_<'c>: Removeable<&'c Card, P>,
{
    fn remove_from(&self, player: &Player<'c>, param: P) -> InnResult<&'c Card> {
        <Board_ as Removeable<&'c Card, P>>::remove(&mut *player.board().borrow_mut(), &param)
            .ok_or(InnovationError::CardNotFound)
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
    fn remove_from(&self, game: &Players<'c>, param: u8) -> InnResult<&'c Card> {
        game.main_card_pile()
            .borrow_mut()
            .remove(&param)
            .ok_or(InnovationError::Win {
                current_player: None,
                situation: WinningSituation::ByScore,
            })
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
