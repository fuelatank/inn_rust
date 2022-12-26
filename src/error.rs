use crate::game::{PlayerId, Players};

#[derive(Debug)]
pub enum WinningSituation {
    SomeOne(PlayerId),
    ByScore,
}

impl WinningSituation {
    pub fn winners(self, game: &Players) -> Vec<PlayerId> {
        match self {
            WinningSituation::SomeOne(p) => vec![p],
            WinningSituation::ByScore => {
                let max_score = game
                    .players_from(0)
                    .map(|player| {
                        // sort order
                        (
                            player.total_score(),
                            player.achievements().clone_inner().len(),
                        )
                    })
                    .max()
                    .unwrap();
                game.players_from(0)
                    .filter_map(|player| {
                        if (
                            player.total_score(),
                            player.achievements().clone_inner().len(),
                        ) == max_score
                        {
                            Some(player.id())
                        } else {
                            None
                        }
                    })
                    .collect()
            }
        }
    }
}

#[derive(Debug)]
pub enum InnovationError {
    ParamUnwrapError,
    CardNotFound,
    InvalidAction,
    WrongPlayerNum,
    Win {
        // the one who executes the most inner card, not share,
        // or the one acting if there's no card in execution
        // just for fun... or to decide which player to start observation?
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
