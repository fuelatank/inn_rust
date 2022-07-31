

pub enum InnovationError {
    ParamUnwrapError,
    CardNotFound,
}

pub type InnResult<T> = Result<T, InnovationError>;