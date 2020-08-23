use amethyst_assets::AssetStorage;
use amethyst_audio::{output::Output, Source, SourceHandle};
use amethyst_core::{
    ecs::*,
    shrev::{EventChannel, ReaderId},
};

use crate::{
    event::{UiEvent, UiEventType::*},
    event_retrigger::{EventRetrigger, EventRetriggerSystem},
    EventReceiver,
};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Provides an `EventRetriggerSystem` that will handle incoming `UiEvent`s
/// and trigger `UiPlaySoundAction`s for entities with attached
/// `UiSoundRetrigger` components.
pub type UiSoundRetriggerSystem = EventRetriggerSystem<UiSoundRetrigger>;

/// Action that will trigger a sound to play in `UiSoundSystem`.
#[derive(Debug, Clone)]
pub struct UiPlaySoundAction(pub SourceHandle);

/// Attach this to an entity to play the respective sound when a `UiEvent`
/// targets the entity.
#[derive(Debug, Clone)]
pub struct UiSoundRetrigger {
    /// The sound that is played when the user begins a click on the entity
    pub on_click_start: Option<UiPlaySoundAction>,
    /// The sound that is played when the user ends a click on the entity
    pub on_click_stop: Option<UiPlaySoundAction>,
    /// The sound that is played when the user starts hovering over the entity
    pub on_hover_start: Option<UiPlaySoundAction>,
    /// The sound that is played when the user stops hovering over the entity
    pub on_hover_stop: Option<UiPlaySoundAction>,
}

impl EventRetrigger for UiSoundRetrigger {
    type In = UiEvent;
    type Out = UiPlaySoundAction;

    fn apply<R>(&self, event: &Self::In, out: &mut R)
    where
        R: EventReceiver<Self::Out>,
    {
        let event_to_trigger = match &event.event_type {
            ClickStart => &self.on_click_start,
            ClickStop => &self.on_click_stop,
            HoverStart => &self.on_hover_start,
            HoverStop => &self.on_hover_stop,
            _ => return,
        };

        if let Some(ev) = event_to_trigger {
            out.receive_one(&ev);
        }
    }
}

/// Handles any dispatches `UiPlaySoundAction`s and plays the received
/// sounds through the set `Output`.
pub fn build_ui_sound_system(mut reader: ReaderId<UiPlaySoundAction>) -> impl Runnable {
    SystemBuilder::new("UiSoundSystem")
        .write_resource::<EventChannel<UiPlaySoundAction>>()
        .read_resource::<AssetStorage<Source>>()
        .write_resource::<Option<Output>>()
        .build(
            move |_commands, world, _resource, (sound_events, audio_storage, audio_output)| {
            for event in sound_events.read(&mut reader) {
                if let Some(output) = audio_output.as_ref() {
                    if let Some(sound) = audio_storage.get(&event.0) {
                        output.play_once(sound, 1.0);
                    }
                }
            }
        }
    )
}

