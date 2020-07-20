use derivative::Derivative;
use winit::Event;

use crate::{
    core::shrev::{EventChannel, ReaderId},
    input::{BindingTypes, InputEvent, StringBindings},
};

#[cfg(feature = "renderer")]
use crate::ui::UiEvent;

/// The enum holding the different types of event that can be received in a `State` in the
/// `handle_event` method.
#[derive(Debug, Derivative)]
#[derivative(Clone(bound = ""))]
pub enum StateEvent<T = StringBindings>
where
    T: BindingTypes,
{
    /// Events sent by the winit window.
    Window(Event),
    /// Events sent by the ui system.
    #[cfg(feature = "renderer")]
    Ui(UiEvent),
    /// Events sent by the input system.
    Input(InputEvent<T>),
}

pub type StateEventChannel<T = StringBindings> = EventChannel<StateEvent<T>>;
