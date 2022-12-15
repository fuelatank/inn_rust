#[derive(Debug)]
pub enum InnovationError {
    ParamUnwrapError,
    CardNotFound,
    InvalidAction,
}

pub type InnResult<T> = Result<T, InnovationError>;
