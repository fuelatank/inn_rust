use crate::{
    action::RefChoice, error::InnResult, game::Players, player::Player, state::ExecutionState,
};
use generator::LocalGenerator;
use serde::Serialize;

pub type GenYield<'c, 'g> = InnResult<ExecutionState<'c, 'g>>;
pub type GenResume<'c, 'g> = RefChoice<'c, 'g>;
pub type FlowState<'c, 'g> = LocalGenerator<'g, GenResume<'c, 'g>, GenYield<'c, 'g>>;

pub type ShareFlow = Box<dyn for<'c, 'g> Fn(&'g Player<'c>, &'g Players<'c>) -> FlowState<'c, 'g>>;
pub type DemandFlow =
    Box<dyn for<'c, 'g> Fn(&'g Player<'c>, &'g Player<'c>, &'g Players<'c>) -> FlowState<'c, 'g>>;

pub enum Dogma {
    Share(ShareFlow),
    Demand(DemandFlow),
}

impl Serialize for Dogma {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Dogma::Share(_) => serializer.serialize_str("share"),
            Dogma::Demand(_) => serializer.serialize_str("demand"),
        }
    }
}
