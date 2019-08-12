use specs::{DispatcherBuilder, World};

/// Initializes a `System` with some interaction with the `World`.
#[typetag::serde]
pub trait DispatcherAction
{
    /// Runs a function on a `DispatcherBuilder` with an access to the `World`.
    /// Meant to provide a way to lazily create a dispatcher from a set of serialized pre-defined
    /// actions.
    /// This has the effect of ultimately allowing dispatchers to be defined from files.
    fn run<'a, 'b>(self, world: &mut World, dispatcher: &mut DispatcherBuilder<'a, 'b>);
}

pub struct InsertSystem {
    pub action: DispatcherAction,
    pub name: String,
    pub deps: Vec<String>,
}
