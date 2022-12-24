use crate::game::PlayerId;

#[derive(Debug)]
pub enum WinningSituation {
    SomeOne(PlayerId),
    ByScore,
    ByExecutor,
}

#[derive(Debug)]
pub enum InnovationError {
    ParamUnwrapError,
    CardNotFound,
    InvalidAction,
    Win {
        current_player: Option<PlayerId>,
        situation: WinningSituation,
    },
}

impl InnovationError {
    pub fn or_set_current_player(self, player: PlayerId) -> Self {
        if let InnovationError::Win {
            current_player: None,
            situation,
        } = self
        {
            // only set winner when current_player is not known,
            // i.e. last in the execution stack
            InnovationError::Win {
                current_player: Some(player),
                situation,
            }
        } else {
            self
        }
    }
}

pub type InnResult<T> = Result<T, InnovationError>;
