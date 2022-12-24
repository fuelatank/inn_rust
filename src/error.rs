use crate::game::PlayerId;

#[derive(Debug)]
pub enum InnovationError {
    ParamUnwrapError,
    CardNotFound,
    InvalidAction,
    Win(Option<PlayerId>), // sometimes we do not know the winner?
}

pub type InnResult<T> = Result<T, InnovationError>;
